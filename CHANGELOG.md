# Changelog

All notable changes to `isaac-sim-rs` are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `omni.isaacsimrs.bridge` Carb extension changelog is in `cpp/omni.isaacsimrs.bridge/docs/CHANGELOG.md`.

## [Unreleased]

## [0.1.0] - 2026-05-01

Initial public release. Unofficial Rust SDK for NVIDIA Isaac Sim, bridging the C++ Carbonite plugin surface and OmniGraph runtime into transport-agnostic Rust consumer adapters (`dora-rs`, `rerun`, custom).

### Added

#### Workspace crates
- `isaac-sim-rs` — top-level facade with cargo features `arrow`, `bridge`, `dora`, `rerun`, `full`. Default features pull only the pure-Rust Arrow decoders (no Isaac Sim install required).
- `carb-sys` — raw FFI bindings to the NVIDIA Carbonite SDK (bindgen, env-driven build).
- `isaac-sim-bridge` — `cxx::bridge` core: thread-safe consumer registry plus `cmd_vel` producer registry. cdylib path gated behind a `cdylib` cargo feature; rlib path compiles without Isaac Sim.
- `isaac-sim-arrow` — Apache Arrow `RecordBatch` conversion for every bridged sensor + `cmd_vel`. Stable schema per sensor, consumer-agnostic.
- `isaac-sim-dora` — dora-rs publisher adapters for every sensor, bidirectional `cmd_vel` adapter, convenience subscribe decoders (`subscribe::*`) for downstream algorithm nodes. Both rlib and cdylib.
- `isaac-sim-rerun` — rerun viewer adapter with per-sensor `RecordingStream`, optional cross-host gRPC, builder API (`Viewer::with_source(...)`).

#### Carb extension
- `omni.isaacsimrs.bridge` Carb extension loads in Isaac Sim 5.1 with full OmniGraph + USD + GSL toolchain (CMake build, packman-fetched USD).
- Custom OmniGraph nodes authored in C++ via the standard `.ogn` codegen, registered eagerly so they appear in OG graphs the moment the extension loads.
- Per-sensor `OgnPublish*ToRust` nodes accept matching NVIDIA RTX / Replicator / Isaac annotator output and forward payload + metadata to Rust via `cxx::bridge`.
- Reverse-direction `OgnApplyCmdVelFromRust` polls a Rust producer slot each tick and emits scalar lin/ang velocities into the Isaac differential controller.
- Runtime adapter selection via `[settings.omni.isaacsimrs.bridge].adapters` (TOML), CLI override via `--/settings/omni.isaacsimrs.bridge/adapters="..."`.
- Per-platform plugin layout: `bin/${platform}/lib*.so`.
- Prebuilt-binary tarball workflow (`just package-extension`).

#### Sensor coverage
Every bridged sensor has Arrow + dora pub/sub + rerun coverage:

| Channel             | Bridge | Arrow | Dora pub | Dora sub | Rerun |
| ------------------- | :----: | :---: | :------: | :------: | :---: |
| LiDAR FlatScan      | x      | x     | x        | x        | x     |
| LiDAR PointCloud    | x      | x     | x        | x        | x     |
| Camera RGB          | x      | x     | x        | x        | x     |
| Camera Depth        | x      | x     | x        | x        | x     |
| Camera Info         | x      | x     | x        | x        | x     |
| IMU                 | x      | x     | x        | x        | x     |
| Chassis Odometry    | x      | x     | x        | x        | x     |
| cmd_vel (Twist)     | x      | x     | x        | x        | x     |

- `cmd_vel` closes the actuation loop: an upstream dora node publishes a Twist; the bridge decodes it and republishes into the producer slot the C++ apply node polls.
- Rerun streams as native primitives (`Points3D`, `Image`, `Pinhole`, `Scalars`, `Transform3D`).

#### Examples
- `examples/lidar-receiver/` — synthesized-input dora pipeline; minimal end-to-end Kit-as-source plus receiver node.
- `examples/nova-carter/` — Nova Carter in a warehouse: 2D + 3D LiDAR, RGB + depth + camera-info, IMU, chassis odometry, cmd_vel apply chain; streams to a rerun viewer over gRPC.
- `examples/nova-carter-dora/` — full dora E2E: Kit publishes 8 channels, receiver decodes all via Arrow shims, emits cmd_vel back; optional rerun gRPC sink. Standalone-buildable.

#### Documentation & tooling
- Per-crate READMEs and rustdoc coverage on the curated public surface.
- `docs/ENV_VARS.md`, `docs/THIRD_PARTY_LICENSES.md`.
- Three audience-tracked README quick-starts: cargo consumer, Isaac Sim user, source build.
- SPDX `MPL-2.0` file headers on all hand-written source files; CI enforcement (`spdx-lint` job).
- CI matrix: cargo `fmt` / `check` / `clippy` / `test` / `doc` / `publish --dry-run`, `cargo-deny` license + advisory gate.
- `cargo-release` driven release workflow.

#### Compatibility (tested on)
- Isaac Sim 5.1.0-rc.19 (Linux x86_64).
- NVIDIA RTX 4090, CUDA 12.6, driver 550.x.
- Ubuntu 24.04, gcc 13.3, CMake 3.28.
- rustc 1.85+ (`rust-version = "1.85"`).
- dora-rs 0.5 (dora-cli, dora-node-api 0.5).
- Apache Arrow 54 (workspace-pinned to match dora).

### Changed
- Workspace version reset from `5.1.0-alpha.0` to `0.1.0` (semver track for the SDK; Isaac Sim target version tracked via `[workspace.metadata].isaac-sim-target`).
- `omni.isaacsimrs.bridge` extension version reset from `5.1.0-alpha.0` to `0.1.0`.
- Public API curated: internal modules demoted to `pub(crate)`; cdylib path gated behind a `cdylib` cargo feature.

### Deprecated
- `ISAAC_SIM_RS_DORA_RUNNER` and `ISAAC_SIM_RS_RERUN_RUNNER` env vars — use the `[settings.omni.isaacsimrs.bridge].adapters` block instead.
