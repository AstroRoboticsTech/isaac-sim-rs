use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::camera::info::{to_record_batch, CameraInfo as ArrowCameraInfo};
use isaac_sim_bridge::{
    register_camera_info_consumer, CameraInfo, CameraInfoFrame, CameraInfoMeta, SourceFilter,
};
use parking_lot::Mutex;

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::DoraPublish;

struct Frame {
    frame_id: String,
    distortion_model: String,
    projection_type: String,
    k: Vec<f64>,
    r: Vec<f64>,
    p: Vec<f64>,
    distortion: Vec<f32>,
    meta: CameraInfoMeta,
}

impl DoraPublish for CameraInfo {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_camera_info_publisher(node, source, output_id);
    }
}

pub fn register_dora_camera_info_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    let (slot, wake) = LatestSlot::<Frame>::new();
    let source_for_drain = source.clone();
    let drain_name = format!("dora-drain-camera_info:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = publish(&node, &output, &frame) {
            log::warn!("[isaac-sim-dora] camera_info publish failed for '{source_for_drain}': {e}");
        }
    });

    register_camera_info_consumer(move |src, info| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            frame_id: info.frame_id.to_string(),
            distortion_model: info.distortion_model.to_string(),
            projection_type: info.projection_type.to_string(),
            k: info.k.to_vec(),
            r: info.r.to_vec(),
            p: info.p.to_vec(),
            distortion: info.distortion.to_vec(),
            meta: *info.meta,
        });
    });
}

pub fn build_struct_array(frame: &CameraInfoFrame<'_>) -> eyre::Result<StructArray> {
    let info = ArrowCameraInfo {
        frame_id: frame.frame_id,
        distortion_model: frame.distortion_model,
        projection_type: frame.projection_type,
        k: frame.k,
        r: frame.r,
        p: frame.p,
        distortion: frame.distortion,
        width: frame.meta.width,
        height: frame.meta.height,
        timestamp_ns: frame.meta.timestamp_ns,
    };
    let batch = to_record_batch(&info)?;
    Ok(StructArray::from(batch))
}

fn publish(node: &Mutex<DoraNode>, output: &DataId, frame: &Frame) -> eyre::Result<()> {
    let view = CameraInfoFrame {
        frame_id: &frame.frame_id,
        distortion_model: &frame.distortion_model,
        projection_type: &frame.projection_type,
        k: &frame.k,
        r: &frame.r,
        p: &frame.p,
        distortion: &frame.distortion,
        meta: &frame.meta,
    };
    let array = build_struct_array(&view)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_camera_info() {
        let k = [500.0_f64, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0];
        let r = [1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let p = [
            500.0_f64, 0.0, 320.0, 0.0, 0.0, 500.0, 240.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let d = [0.0_f32; 5];
        let meta = CameraInfoMeta {
            width: 640,
            height: 480,
            timestamp_ns: 11,
        };
        let view = CameraInfoFrame {
            frame_id: "sim_camera",
            distortion_model: "plumb_bob",
            projection_type: "pinhole",
            k: &k,
            r: &r,
            p: &p,
            distortion: &d,
            meta: &meta,
        };
        let array = build_struct_array(&view).expect("build");
        assert_eq!(array.num_columns(), 10);
        assert_eq!(array.len(), 1);
    }
}
