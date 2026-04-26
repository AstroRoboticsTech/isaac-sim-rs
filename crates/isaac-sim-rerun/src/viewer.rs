use std::env;

use rerun::{RecordingStream, RecordingStreamBuilder};

use crate::lidar::register_rerun_lidar_publisher;

const APP_ID: &str = "isaac-sim-rs";
const GRPC_ADDR_ENV: &str = "ISAAC_SIM_RS_RERUN_GRPC_ADDR";
const DEFAULT_GRPC_ADDR: &str = "127.0.0.1:9876";

type BlueprintFn = Box<dyn FnOnce(&RecordingStream) -> eyre::Result<()>>;

/// Pure config until `.run()`; no I/O happens earlier.
///
/// ```no_run
/// use isaac_sim_rerun::Viewer;
///
/// Viewer::new()
///     .with_grpc_addr("192.168.1.10:9876")
///     .with_lidar("/World/LidarGraph/LidarFwd", "scene/lidar/scan")
///     .with_blueprint(|rec| {
///         rec.log_static("scene/lidar/scan", &rerun::TextDocument::new("hello"))?;
///         Ok(())
///     })
///     .run()?;
/// # Ok::<(), eyre::Report>(())
/// ```
#[derive(Default)]
pub struct Viewer {
    grpc_addr: Option<String>,
    lidars: Vec<(String, String)>,
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

    pub fn with_lidar(mut self, source: impl Into<String>, entity_path: impl Into<String>) -> Self {
        self.lidars.push((source.into(), entity_path.into()));
        self
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

        for (source, entity_path) in self.lidars {
            log::info!("[isaac-sim-rerun] lidar: '{source}' -> '{entity_path}'");
            register_rerun_lidar_publisher(rec.clone(), source, entity_path);
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
            .with_lidar("/A", "a")
            .with_lidar("/B", "b");
        assert_eq!(v.grpc_addr.as_deref(), Some("10.0.0.1:1234"));
        assert_eq!(v.lidars.len(), 2);
        assert_eq!(v.lidars[0], ("/A".into(), "a".into()));
        assert_eq!(v.lidars[1], ("/B".into(), "b".into()));
        assert!(v.blueprint.is_none());
    }

    #[test]
    fn builder_stores_blueprint_closure() {
        let v = Viewer::new().with_blueprint(|_rec| Ok(()));
        assert!(v.blueprint.is_some());
    }
}
