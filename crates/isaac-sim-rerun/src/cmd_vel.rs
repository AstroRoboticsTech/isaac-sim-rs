use isaac_sim_bridge::{register_cmd_vel_consumer, CmdVel, CmdVelChannel};
use rerun::{RecordingStream, Scalars};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

impl RerunRender for CmdVelChannel {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_cmd_vel_publisher(rec, source, entity_path);
    }
}

pub fn log_cmd_vel(rec: &RecordingStream, entity_path: &str, twist: &CmdVel) -> eyre::Result<()> {
    rec.log(
        format!("{entity_path}/linear/x"),
        &Scalars::single(twist.linear_x as f64),
    )?;
    rec.log(
        format!("{entity_path}/linear/y"),
        &Scalars::single(twist.linear_y as f64),
    )?;
    rec.log(
        format!("{entity_path}/linear/z"),
        &Scalars::single(twist.linear_z as f64),
    )?;
    rec.log(
        format!("{entity_path}/angular/x"),
        &Scalars::single(twist.angular_x as f64),
    )?;
    rec.log(
        format!("{entity_path}/angular/y"),
        &Scalars::single(twist.angular_y as f64),
    )?;
    rec.log(
        format!("{entity_path}/angular/z"),
        &Scalars::single(twist.angular_z as f64),
    )?;
    Ok(())
}

pub fn register_rerun_cmd_vel_publisher(rec: RecordingStream, source: String, entity_path: String) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<CmdVel>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-cmd_vel:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |twist| {
        if let Err(e) = log_cmd_vel(&rec, &entity_path_for_drain, &twist) {
            log::warn!(
                "[isaac-sim-rerun] cmd_vel log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_cmd_vel_consumer(move |target, twist| {
        if !filter.matches(target) {
            return;
        }
        slot.publish(*twist);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn fake_twist() -> CmdVel {
        CmdVel {
            linear_x: 0.4,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.3,
            timestamp_ns: 1,
        }
    }

    #[test]
    fn log_cmd_vel_writes_to_memory_sink() {
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_cmd_vel(&rec, "chassis/cmd_vel", &fake_twist()).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }

    #[test]
    fn register_rerun_cmd_vel_publisher_drains_to_storage() {
        use isaac_sim_bridge::register_cmd_vel_producer;

        let target = "/test/articulation/rerun_e2e";
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-cmdvel-e2e")
            .memory()
            .expect("memory recording");
        register_rerun_cmd_vel_publisher(
            rec.clone(),
            target.to_string(),
            "chassis/cmd_vel".to_string(),
        );
        let slot = register_cmd_vel_producer(target);
        slot.publish(fake_twist());
        for _ in 0..50 {
            rec.flush_blocking().expect("flush");
            let msgs = storage.take();
            if !msgs.is_empty() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("no Scalars logged to memory sink within 500 ms");
    }
}
