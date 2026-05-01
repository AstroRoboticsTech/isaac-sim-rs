# Changelog — omni.isaacsimrs.bridge

The extension's Carb-side changes. The Rust SDK changelog is at the repo root.

## [Unreleased]

### Added
- `[settings.omni.isaacsimrs.bridge].adapters` runtime adapter selection.
- Per-platform plugin layout (`bin/${platform}/lib*.so`).
- `install(...)` CMake rules for prebuilt-tarball distribution.
- `OgnApplyCmdVelFromRust` reverse-direction actuator node.

### Changed
- Extension version moved from `5.1.0-alpha.0` to `0.1.0`.
- Category changed from `Internal` to `Sensor` for public registry visibility.

### Deprecated
- `ISAAC_SIM_RS_DORA_RUNNER` and `ISAAC_SIM_RS_RERUN_RUNNER` env vars.

## [0.1.0] - TBD

Initial public release.
