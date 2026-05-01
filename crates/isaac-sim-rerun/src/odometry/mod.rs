// SPDX-License-Identifier: MPL-2.0
use isaac_sim_bridge::{register_odometry_consumer, Odometry, OdometryMeta};
use rerun::{Quaternion, RecordingStream, Scalars, Transform3D};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

/// One odometry sample handed off to the drain thread.
struct Frame {
    meta: OdometryMeta,
}

impl RerunRender for Odometry {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_odometry_publisher(rec, source, entity_path);
    }
}

pub fn log_odometry(
    rec: &RecordingStream,
    entity_path: &str,
    meta: &OdometryMeta,
) -> eyre::Result<()> {
    let translation = [
        meta.position_x as f32,
        meta.position_y as f32,
        meta.position_z as f32,
    ];
    let quat = Quaternion::from_wxyz([
        meta.orientation_w as f32,
        meta.orientation_x as f32,
        meta.orientation_y as f32,
        meta.orientation_z as f32,
    ]);
    // Logging Transform3D on the chassis entity moves all child
    // entities (lidar_*, camera_*, imu) in the spatial view — so the
    // robot visibly drives through the warehouse with sensors attached.
    let xform = Transform3D::from_translation_rotation(translation, quat);
    rec.log(entity_path, &xform)?;

    rec.log(
        format!("{entity_path}/lin_vel/x"),
        &Scalars::single(meta.lin_vel_x),
    )?;
    rec.log(
        format!("{entity_path}/lin_vel/y"),
        &Scalars::single(meta.lin_vel_y),
    )?;
    rec.log(
        format!("{entity_path}/lin_vel/z"),
        &Scalars::single(meta.lin_vel_z),
    )?;
    rec.log(
        format!("{entity_path}/ang_vel/x"),
        &Scalars::single(meta.ang_vel_x),
    )?;
    rec.log(
        format!("{entity_path}/ang_vel/y"),
        &Scalars::single(meta.ang_vel_y),
    )?;
    rec.log(
        format!("{entity_path}/ang_vel/z"),
        &Scalars::single(meta.ang_vel_z),
    )?;
    Ok(())
}

pub fn register_rerun_odometry_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<Frame>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-odometry:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = log_odometry(&rec, &entity_path_for_drain, &frame.meta) {
            log::warn!(
                "[isaac-sim-rerun] log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_odometry_consumer(move |src, _chassis_id, _odom_id, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame { meta: *meta });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_meta() -> OdometryMeta {
        OdometryMeta {
            position_x: 1.0,
            position_y: 2.0,
            position_z: 0.0,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            lin_vel_x: 0.5,
            lin_vel_y: 0.0,
            lin_vel_z: 0.0,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.1,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn log_odometry_writes_to_memory_sink() {
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_odometry(&rec, "chassis", &fake_meta()).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }
}
