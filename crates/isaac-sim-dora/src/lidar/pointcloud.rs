use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::lidar::pointcloud::{to_record_batch, LidarPointCloud as ArrowPointCloud};
use isaac_sim_bridge::{
    register_lidar_pointcloud_consumer, LidarPointCloud, LidarPointCloudMeta, SourceFilter,
};
use parking_lot::Mutex;

use crate::sensor::DoraPublish;

impl DoraPublish for LidarPointCloud {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_lidar_pointcloud_publisher(node, source, output_id);
    }
}

pub fn register_dora_lidar_pointcloud_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    register_lidar_pointcloud_consumer(move |src, points, meta| {
        if !filter.matches(src) {
            return;
        }
        if let Err(e) = publish(&node, &output, points, meta) {
            log::warn!("[isaac-sim-dora] lidar_pointcloud publish failed for '{source}': {e}");
        }
    });
}

/// Pure conversion from bridge-side packed-XYZ points + meta to a
/// dora-ready Arrow StructArray. Extracted for testability without a
/// DoraNode.
pub fn build_struct_array(points: &[f32], meta: &LidarPointCloudMeta) -> eyre::Result<StructArray> {
    let pc = ArrowPointCloud {
        points,
        num_points: meta.num_points,
        width: meta.width,
        height: meta.height,
    };
    let batch = to_record_batch(&pc)?;
    Ok(StructArray::from(batch))
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    points: &[f32],
    meta: &LidarPointCloudMeta,
) -> eyre::Result<()> {
    let array = build_struct_array(points, meta)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_points() {
        let points = [
            1.0_f32, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            0.0, 0.0, 1.0,
        ];
        let meta = LidarPointCloudMeta {
            num_points: 3,
            width: 3,
            height: 1,
        };
        let array = build_struct_array(&points, &meta).expect("build");
        assert_eq!(array.num_columns(), 4);
        assert_eq!(array.len(), 1);
    }
}
