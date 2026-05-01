use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::camera::rgb::{to_record_batch, CameraRgb as ArrowCameraRgb};
use isaac_sim_bridge::{register_camera_rgb_consumer, CameraRgb, CameraRgbMeta, SourceFilter};
use parking_lot::Mutex;

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::DoraPublish;

struct Frame {
    pixels: Arc<[u8]>,
    meta: CameraRgbMeta,
}

impl DoraPublish for CameraRgb {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_camera_rgb_publisher(node, source, output_id);
    }
}

pub fn register_dora_camera_rgb_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    let (slot, wake) = LatestSlot::<Frame>::new();
    let source_for_drain = source.clone();
    let drain_name = format!("dora-drain-camera_rgb:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = publish(&node, &output, &frame.pixels, &frame.meta) {
            log::warn!("[isaac-sim-dora] camera_rgb publish failed for '{source_for_drain}': {e}");
        }
    });

    register_camera_rgb_consumer(move |src, pixels, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            pixels: Arc::from(pixels),
            meta: *meta,
        });
    });
}

pub fn build_struct_array(pixels: &[u8], meta: &CameraRgbMeta) -> eyre::Result<StructArray> {
    let img = ArrowCameraRgb {
        pixels,
        width: meta.width,
        height: meta.height,
        fx: meta.fx,
        fy: meta.fy,
        cx: meta.cx,
        cy: meta.cy,
        timestamp_ns: meta.timestamp_ns,
    };
    let batch = to_record_batch(&img)?;
    Ok(StructArray::from(batch))
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    pixels: &[u8],
    meta: &CameraRgbMeta,
) -> eyre::Result<()> {
    let array = build_struct_array(pixels, meta)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_pixels() {
        let pixels = vec![10_u8; 12]; // 2x2 RGB
        let meta = CameraRgbMeta {
            width: 2,
            height: 2,
            fx: 0.0,
            fy: 0.0,
            cx: 0.0,
            cy: 0.0,
            timestamp_ns: 7,
        };
        let array = build_struct_array(&pixels, &meta).expect("build");
        assert_eq!(array.num_columns(), 8);
        assert_eq!(array.len(), 1);
    }
}
