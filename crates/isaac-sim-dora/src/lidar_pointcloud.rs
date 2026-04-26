use std::sync::{Arc, Mutex};

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::lidar_pointcloud::{to_record_batch, LidarPointCloud};
use isaac_sim_bridge::{register_lidar_pointcloud_consumer, LidarPointCloudMeta};

pub fn register_dora_lidar_pointcloud_publisher(
    node: Arc<Mutex<DoraNode>>,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();

    register_lidar_pointcloud_consumer(move |azimuth, elevation, distance, intensity, meta| {
        if let Err(e) = publish(
            &node, &output, azimuth, elevation, distance, intensity, meta,
        ) {
            log::warn!("[isaac-sim-dora] lidar_pointcloud publish failed: {e}");
        }
    });
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    azimuth: &[f32],
    elevation: &[f32],
    distance: &[f32],
    intensity: &[f32],
    meta: &LidarPointCloudMeta,
) -> eyre::Result<()> {
    let pc = LidarPointCloud {
        azimuth,
        elevation,
        distance,
        intensity,
        num_points: meta.num_points,
    };
    let batch = to_record_batch(&pc)?;
    let array = StructArray::from(batch);

    let mut guard = node
        .lock()
        .map_err(|_| eyre::eyre!("dora node mutex poisoned"))?;
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}
