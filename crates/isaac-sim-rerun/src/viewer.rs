// SPDX-License-Identifier: MPL-2.0
//! The `Viewer` builder: configure sensor subscriptions, then call `run()` to stream to rerun.
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};
use rerun::{RecordingStream, RecordingStreamBuilder};

use crate::sensor::RerunRender;

const APP_ID: &str = "isaac-sim-rs";
const GRPC_ADDR_ENV: &str = "ISAAC_SIM_RS_RERUN_GRPC_ADDR";
const DEFAULT_GRPC_ADDR: &str = "127.0.0.1:9876";

type BlueprintFn = Box<dyn FnOnce(&RecordingStream) -> eyre::Result<()>>;
type Bind = Box<dyn FnOnce(RecordingStream)>;

/// Pure config until `.run()`; no I/O happens earlier.
///
/// `with_source::<S>(source, entity)` registers a source-filtered
/// consumer for any sensor `S: RerunRender`. Multiple sensors of the
/// same type co-exist by chaining more `with_source` calls. Convenience
/// `with_lidar_flatscan` / `with_lidar_pointcloud` shims call through.
///
/// ```no_run
/// use isaac_sim_rerun::Viewer;
/// use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};
///
/// Viewer::new()
///     .with_grpc_addr("192.168.1.10:9876")
///     .with_source(LidarFlatScan, "/World/Carter/lidar_2d", "scene/lidar/flatscan")
///     .with_source(LidarPointCloud, "/World/Carter/.../PandarXT", "scene/lidar/pointcloud")
///     .run()?;
/// # Ok::<(), eyre::Report>(())
/// ```
#[derive(Default)]
pub struct Viewer {
    grpc_addr: Option<String>,
    binds: Vec<Bind>,
    blueprint: Option<BlueprintFn>,
}

impl Viewer {
    /// Create a builder with no sensors subscribed and the default gRPC address.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the rerun gRPC endpoint. Default: `127.0.0.1:9876` (or
    /// `ISAAC_SIM_RS_RERUN_GRPC_ADDR` env var if set at `run()` time).
    pub fn with_grpc_addr(mut self, addr: impl Into<String>) -> Self {
        self.grpc_addr = Some(addr.into());
        self
    }

    /// Register a rerun publisher for sensor `S`. Each call adds an
    /// independent source-filtered subscription. At `run()` time each
    /// `with_source` call gets its own `RecordingStream` over its own
    /// gRPC connection so a high-bandwidth sensor (camera) cannot
    /// backpressure a low-bandwidth one (LiDAR). All streams share one
    /// `recording_id`, so the viewer renders them on a single timeline.
    pub fn with_source<S: RerunRender>(
        mut self,
        _sensor: S,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let entity_path = entity_path.into();
        let label = format!("{}: '{source}' -> '{entity_path}'", S::NAME);
        self.binds.push(Box::new(move |rec: RecordingStream| {
            log::info!("[isaac-sim-rerun] {label}");
            S::register(rec, source, entity_path);
        }));
        self
    }

    /// Convenience shorthand for `with_source(LidarFlatScan, source, entity_path)`.
    pub fn with_lidar_flatscan(
        self,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        self.with_source(LidarFlatScan, source, entity_path)
    }

    /// Convenience shorthand for `with_source(LidarPointCloud, source, entity_path)`.
    pub fn with_lidar_pointcloud(
        self,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        self.with_source(LidarPointCloud, source, entity_path)
    }

    /// Attach a blueprint closure that runs on the first sensor's `RecordingStream`
    /// before any frame is forwarded. Use to set up rerun blueprint layout.
    pub fn with_blueprint<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&RecordingStream) -> eyre::Result<()> + 'static,
    {
        self.blueprint = Some(Box::new(f));
        self
    }

    /// Open gRPC connections and register all configured sensor consumers. Blocks
    /// until the process exits (the bridge thread drives the consumer callbacks).
    pub fn run(self) -> eyre::Result<()> {
        let addr = self.grpc_addr.unwrap_or_else(|| {
            env::var(GRPC_ADDR_ENV).unwrap_or_else(|_| DEFAULT_GRPC_ADDR.to_string())
        });
        let url = format!("rerun+http://{addr}/proxy");
        let recording_id = recording_id();
        log::info!("[isaac-sim-rerun] connecting to {url} (recording_id={recording_id})");

        // Per-sensor RecordingStream — independent gRPC connections to
        // the same recording_id. Camera bandwidth no longer shares a
        // queue with LiDAR. The blueprint (if any) goes onto the first
        // sensor's stream rather than a dedicated one — flush_blocking
        // on a freshly-connected stream blocks the runner's static-init
        // path indefinitely (rerun's gRPC client establishes lazily),
        // and a dedicated short-lived blueprint stream tripped that hang.
        let mut blueprint = self.blueprint;
        for bind in self.binds {
            let rec = build_stream(&recording_id, &url)?;
            if let Some(bp) = blueprint.take() {
                bp(&rec)?;
            }
            bind(rec);
        }
        Ok(())
    }
}

fn build_stream(recording_id: &str, url: &str) -> eyre::Result<RecordingStream> {
    Ok(RecordingStreamBuilder::new(APP_ID)
        .recording_id(recording_id)
        .connect_grpc_opts(url.to_string())?)
}

fn recording_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("isaac-sim-rs-{nanos}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_collects_lidar_subscriptions() {
        let v = Viewer::new()
            .with_grpc_addr("10.0.0.1:1234")
            .with_lidar_flatscan("/A", "a")
            .with_lidar_pointcloud("/B", "b")
            .with_lidar_pointcloud("/C", "c");
        assert_eq!(v.grpc_addr.as_deref(), Some("10.0.0.1:1234"));
        assert_eq!(v.binds.len(), 3);
        assert!(v.blueprint.is_none());
    }

    #[test]
    fn builder_stores_blueprint_closure() {
        let v = Viewer::new().with_blueprint(|_rec| Ok(()));
        assert!(v.blueprint.is_some());
    }

    #[test]
    fn recording_id_is_stable_per_call_format() {
        let id = recording_id();
        assert!(id.starts_with("isaac-sim-rs-"));
        assert!(id.len() > "isaac-sim-rs-".len());
    }
}
