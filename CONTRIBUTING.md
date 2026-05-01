# Contributing to isaac-sim-rs

`isaac-sim-rs` is maintained by Astro Robotics. Patches welcome via GitHub PR.

## Dev environment

See the "Source build" track in the [project README](README.md) for `ISAAC_SIM` env-var setup, packman fetch, and `just build` flow. For pure-Rust changes (schema layer, Arrow codecs, dora/rerun adapters) no Isaac Sim install is required — `cargo test --workspace --lib` covers the unit tests.

## Per-file SPDX headers

Every hand-written source file (`.rs`, `.cpp`, `.hpp`, `.h`, shell, Python, justfile, YAML) starts with `// SPDX-License-Identifier: MPL-2.0` (or `#` variant for hash-comment files). New files added in your PR must follow. CI's `spdx-lint` job enforces this.

## Comment policy

Comments document WHY — subtle invariants, surprising behaviour, non-obvious constraints. Don't restate WHAT; the code already shows it. One short line cap; no multi-paragraph rustdoc preambles. No tracking IDs (`// see #C12`, `// fixes issue 42`) in source or config files. Tracking docs (`AUDIT.md`, `SHIP_AUDIT.md`) may use IDs; code and configs must not.

## Test runs

```
cargo test --workspace --lib --all-features
cargo test --doc --workspace
```

The C++ extension smoke test (`omni.kit.test` in `cpp/omni.isaacsimrs.bridge/tests/`) requires a local Isaac Sim install. Run it with `just kit-smoke` after sourcing the Isaac Sim environment.

## Release flow

Tag `vX.Y.Z` on `main`; CI publishes the workspace in dependency order (see [`release.toml`](release.toml) and [`.github/workflows/release.yml`](.github/workflows/release.yml)).

Bumping the version: update `version` in the workspace `Cargo.toml`, then run `cargo-release` with the appropriate level. The `pre-release-replacements` in `release.toml` update `CHANGELOG.md` automatically.

## Sign your commits

DCO: every commit needs a `Signed-off-by` trailer.

```
git commit -s -m "your message"
```

No CLA required.

## Reporting bugs

Open a GitHub issue at <https://github.com/AstroRoboticsTech/isaac-sim-rs/issues>. Include Isaac Sim version, OS, GPU driver, and a minimal reproduction if possible.
