use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::lidar::flatscan::{to_record_batch, LidarFlatScan as ArrowFlatScan};
use isaac_sim_bridge::{
    register_lidar_flatscan_consumer, LidarFlatScan, LidarFlatScanMeta, SourceFilter,
};
use parking_lot::Mutex;

use crate::sensor::DoraPublish;

impl DoraPublish for LidarFlatScan {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_lidar_flatscan_publisher(node, source, output_id);
    }
}

pub fn register_dora_lidar_flatscan_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    register_lidar_flatscan_consumer(move |src, scan, intensities, meta| {
        if !filter.matches(src) {
            return;
        }
        if let Err(e) = publish(&node, &output, scan, intensities, meta) {
            log::warn!("[isaac-sim-dora] lidar_flatscan publish failed for '{source}': {e}");
        }
    });
}

/// Pure conversion from bridge-side scan + meta to a dora-ready Arrow
/// StructArray. Extracted so it's testable without spinning up a
/// DoraNode.
pub fn build_struct_array(
    scan: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) -> eyre::Result<StructArray> {
    let lidar = ArrowFlatScan {
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
    Ok(StructArray::from(batch))
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    scan: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) -> eyre::Result<()> {
    let array = build_struct_array(scan, intensities, meta)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    fn fake_meta() -> LidarFlatScanMeta {
        LidarFlatScanMeta {
            horizontal_fov: 270.0,
            horizontal_resolution: 0.25,
            azimuth_min: -135.0,
            azimuth_max: 135.0,
            depth_min: 0.1,
            depth_max: 30.0,
            num_rows: 1,
            num_cols: 4,
            rotation_rate: 10.0,
        }
    }

    #[test]
    fn build_struct_array_round_trips_scan() {
        let scan = [0.5_f32, 1.2, 2.7, 3.0];
        let intensities = [10_u8, 50, 200, 100];
        let array = build_struct_array(&scan, &intensities, &fake_meta()).expect("build");
        // 11-column flatscan schema; 1 row per dispatch.
        assert_eq!(array.num_columns(), 11);
        assert_eq!(array.len(), 1);
    }
}
