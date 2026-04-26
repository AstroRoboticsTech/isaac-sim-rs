use std::sync::{Arc, Mutex};

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::lidar_flatscan::{to_record_batch, LidarFlatScan};
use isaac_sim_bridge::{register_lidar_flatscan_consumer, LidarFlatScanMeta};

pub fn register_dora_lidar_flatscan_publisher(
    node: Arc<Mutex<DoraNode>>,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();

    register_lidar_flatscan_consumer(move |scan, intensities, meta| {
        if let Err(e) = publish(&node, &output, scan, intensities, meta) {
            log::warn!("[isaac-sim-dora] lidar_flatscan publish failed: {e}");
        }
    });
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    scan: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) -> eyre::Result<()> {
    let lidar = LidarFlatScan {
        depths: scan,
        intensities,
        horizontal_fov: meta.horizontal_fov,
        horizontal_resolution: meta.horizontal_resolution,
        azimuth_min: meta.azimuth_min,
        azimuth_max: meta.azimuth_max,
        depth_min: meta.depth_min,
        depth_max: meta.depth_max,
        num_rows: meta.num_rows,
        num_cols: meta.num_cols,
        rotation_rate: meta.rotation_rate,
    };
    let batch = to_record_batch(&lidar)?;
    let array = StructArray::from(batch);

    let mut guard = node
        .lock()
        .map_err(|_| eyre::eyre!("dora node mutex poisoned"))?;
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}
