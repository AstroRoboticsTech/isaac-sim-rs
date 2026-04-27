use std::env;

use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};
use rerun::{RecordingStream, RecordingStreamBuilder};

use crate::sensor::RerunRender;

const APP_ID: &str = "isaac-sim-rs";
const GRPC_ADDR_ENV: &str = "ISAAC_SIM_RS_RERUN_GRPC_ADDR";
const DEFAULT_GRPC_ADDR: &str = "127.0.0.1:9876";

type BlueprintFn = Box<dyn FnOnce(&RecordingStream) -> eyre::Result<()>>;
type Bind = Box<dyn FnOnce(&RecordingStream)>;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grpc_addr(mut self, addr: impl Into<String>) -> Self {
        self.grpc_addr = Some(addr.into());
        self
    }

    /// Register a rerun publisher for sensor `S`. Each call adds an
    /// independent source-filtered subscription.
    pub fn with_source<S: RerunRender>(
        mut self,
        _sensor: S,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let entity_path = entity_path.into();
        let label = format!("{}: '{source}' -> '{entity_path}'", S::NAME);
        self.binds.push(Box::new(move |rec: &RecordingStream| {
            log::info!("[isaac-sim-rerun] {label}");
            S::register(rec.clone(), source, entity_path);
        }));
        self
    }

    pub fn with_lidar_flatscan(
        self,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        self.with_source(LidarFlatScan, source, entity_path)
    }

    pub fn with_lidar_pointcloud(
        self,
        source: impl Into<String>,
        entity_path: impl Into<String>,
    ) -> Self {
        self.with_source(LidarPointCloud, source, entity_path)
    }

    pub fn with_blueprint<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&RecordingStream) -> eyre::Result<()> + 'static,
    {
        self.blueprint = Some(Box::new(f));
        self
    }

    pub fn run(self) -> eyre::Result<()> {
        let addr = self.grpc_addr.unwrap_or_else(|| {
            env::var(GRPC_ADDR_ENV).unwrap_or_else(|_| DEFAULT_GRPC_ADDR.to_string())
        });
        let url = format!("rerun+http://{addr}/proxy");
        log::info!("[isaac-sim-rerun] connecting to {url}");
        let rec = RecordingStreamBuilder::new(APP_ID).connect_grpc_opts(url)?;

        if let Some(bp) = self.blueprint {
            bp(&rec)?;
        }

        for bind in self.binds {
            bind(&rec);
        }
        Ok(())
    }
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
}
