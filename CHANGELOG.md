# Changelog

All notable changes to `isaac-sim-rs` are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `omni.isaacsimrs.bridge` Carb extension changelog is in `cpp/omni.isaacsimrs.bridge/docs/CHANGELOG.md`.

## [Unreleased]

### Added
- `isaac-sim-rs` facade crate with cargo features (`arrow`, `bridge`, `dora`, `rerun`, `full`).
- Per-crate READMEs and rustdoc coverage on the curated public surface.
- Kit extension prebuilt-binary tarball workflow (`just package-extension`).
- `[settings.omni.isaacsimrs.bridge].adapters` runtime adapter selection (replacing the deprecated `ISAAC_SIM_RS_<A>_RUNNER` env vars).
- `docs/ENV_VARS.md`, `docs/THIRD_PARTY_LICENSES.md`, expanded CI matrix.
- SPDX `MPL-2.0` file headers on all hand-written source files.

### Changed
- Workspace version moved from `5.1.0-alpha.0` to `0.1.0` (semver track for the SDK; Isaac Sim target version tracked via `[workspace.metadata].isaac-sim-target`).
- `omni.isaacsimrs.bridge` extension version moved from `5.1.0-alpha.0` to `0.1.0`.
- Plugin per-platform layout moved to `bin/${platform}/lib*.so`.
- Public API curated: internal modules demoted to `pub(crate)`; cdylib path gated behind a `cdylib` cargo feature.

### Deprecated
- `ISAAC_SIM_RS_DORA_RUNNER` and `ISAAC_SIM_RS_RERUN_RUNNER` env vars (use the `[settings].adapters` block).

## [0.1.0] - TBD

Initial public release. See [Unreleased] for the full list.
