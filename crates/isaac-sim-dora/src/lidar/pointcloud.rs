use std::sync::{Arc, Mutex};

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::lidar::pointcloud::{to_record_batch, LidarPointCloud};
use isaac_sim_bridge::{register_lidar_pointcloud_consumer, LidarPointCloudMeta, SourceFilter};

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

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    points: &[f32],
    meta: &LidarPointCloudMeta,
) -> eyre::Result<()> {
    let pc = LidarPointCloud {
        points,
        num_points: meta.num_points,
        width: meta.width,
        height: meta.height,
    };
    let batch = to_record_batch(&pc)?;
    let array = StructArray::from(batch);

    let mut guard = node
        .lock()
        .map_err(|_| eyre::eyre!("dora node mutex poisoned"))?;
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}
