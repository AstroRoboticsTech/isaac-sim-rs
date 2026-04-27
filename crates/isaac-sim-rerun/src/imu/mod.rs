use isaac_sim_bridge::{register_imu_consumer, Imu, ImuMeta};
use rerun::{Quaternion, RecordingStream, Scalars, Transform3D};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

/// IMU sample handed off to the drain thread.
struct Frame {
    meta: ImuMeta,
}

impl RerunRender for Imu {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_imu_publisher(rec, source, entity_path);
    }
}

pub fn log_imu(
    rec: &RecordingStream,
    entity_path: &str,
    meta: &ImuMeta,
) -> eyre::Result<()> {
    rec.log(
        format!("{entity_path}/lin_acc/x"),
        &Scalars::single(meta.lin_acc_x),
    )?;
    rec.log(
        format!("{entity_path}/lin_acc/y"),
        &Scalars::single(meta.lin_acc_y),
    )?;
    rec.log(
        format!("{entity_path}/lin_acc/z"),
        &Scalars::single(meta.lin_acc_z),
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
    let quat = Quaternion::from_wxyz([
        meta.orientation_w as f32,
        meta.orientation_x as f32,
        meta.orientation_y as f32,
        meta.orientation_z as f32,
    ]);
    let xform = Transform3D::from_rotation(quat);
    rec.log(entity_path, &xform)?;
    Ok(())
}

pub fn register_rerun_imu_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<Frame>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-imu:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = log_imu(&rec, &entity_path_for_drain, &frame.meta) {
            log::warn!(
                "[isaac-sim-rerun] log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_imu_consumer(move |src, _frame_id, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame { meta: *meta });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_meta() -> ImuMeta {
        ImuMeta {
            lin_acc_x: 0.0,
            lin_acc_y: 0.0,
            lin_acc_z: 9.81,
            ang_vel_x: 0.1,
            ang_vel_y: 0.2,
            ang_vel_z: 0.3,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn log_imu_writes_to_memory_sink() {
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_imu(&rec, "scene/imu", &fake_meta()).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }

    #[test]
    fn log_imu_quaternion_components_propagate_in_xyzw_order() {
        let q = Quaternion::from_wxyz([1.0, 2.0, 3.0, 4.0]);
        let xyzw = q.xyzw();
        // wxyz=(1,2,3,4) → xyzw=(2,3,4,1) per from_wxyz contract
        assert_eq!(xyzw, [2.0, 3.0, 4.0, 1.0]);
    }
}
