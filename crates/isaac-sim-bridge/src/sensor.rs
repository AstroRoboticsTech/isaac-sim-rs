/// Type-level identifier for a sensor domain.
///
/// `NAME` simultaneously plays four roles:
///
/// 1. **Registry key** — the consumer-registry insert/lookup key so the
///    bridge can fan out callbacks per sensor without a runtime string map.
/// 2. **Env-var prefix** — adapters derive `ISAAC_SIM_RS_DORA_<NAME>_*` and
///    `ISAAC_SIM_RS_RERUN_<NAME>_*` variable names from it.
/// 3. **Log label** — structured log lines use `S::NAME` so every sensor's
///    messages are grep-able by name.
/// 4. **Default dora id** — `register_publisher` defaults both OUTPUT and
///    the dora node id to `S::NAME` when the env var is unset.
///
/// If a fifth use case arises that needs a value that differs from NAME
/// for any existing sensor, introduce a separate associated const or a
/// distinct trait rather than adding another load to this one. The existing
/// exception is `CmdVelChannel`: its publisher output defaults to
/// `"cmd_vel_observed"` (not `S::NAME`) via `register_publisher_with_default`
/// because role 4 conflicts with role 2 when the subscriber's INPUT also
/// defaults to `"cmd_vel"`.
///
/// Implemented by ZST markers in each sensor module (e.g. `LidarFlatScan`,
/// `LidarPointCloud`). Adapters layer their own per-sensor trait on top
/// (`RerunRender`, `DoraPublish`) keyed on these markers.
pub trait Sensor: 'static {
    const NAME: &'static str;
}
