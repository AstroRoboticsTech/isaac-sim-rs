# Third-party licenses

`isaac-sim-rs` and `omni.isaacsimrs.bridge` link or bundle the following third-party libraries.

## Rust dependencies (linked into publishable crates)

| Crate | Version | License | Used by |
| --- | --- | --- | --- |
| arrow | 54 | Apache-2.0 | isaac-sim-arrow, isaac-sim-dora |
| arc-swap | 1 | MIT OR Apache-2.0 | isaac-sim-bridge, isaac-sim-dora, isaac-sim-rerun |
| bindgen | 0.70 | BSD-3-Clause | carb-sys (build-dep only, not shipped) |
| bytemuck | 1 | MIT OR Apache-2.0 OR Zlib | isaac-sim-rerun (dev + direct) |
| cxx | 1 | MIT OR Apache-2.0 | isaac-sim-bridge |
| cxx-build | 1 | MIT OR Apache-2.0 | isaac-sim-bridge (build-dep only) |
| dora-node-api | 0.5 | Apache-2.0 | isaac-sim-dora |
| env_logger | 0.11 | MIT OR Apache-2.0 | isaac-sim-bridge |
| eyre | 0.6 | MIT OR Apache-2.0 | isaac-sim-dora, isaac-sim-rerun |
| log | 0.4 | MIT OR Apache-2.0 | all crates |
| parking_lot | 0.12 | MIT OR Apache-2.0 | isaac-sim-bridge, isaac-sim-dora |
| rerun | 0.31 | MIT OR Apache-2.0 | isaac-sim-rerun |
| serde_json | 1 | MIT OR Apache-2.0 | isaac-sim-bridge (dev-dep) |

Each crate listed above carries its own license text, which `cargo` vendors alongside the source. The SPDX expressions are dual-licenses — the consumer may choose either.

`bindgen` and `cxx-build` are build-time tools only; they are not linked into any shipping binary.

## C++ and system dependencies (linked by the Carb extension)

| Library | License | Notes |
| --- | --- | --- |
| NVIDIA Carbonite SDK | NVIDIA proprietary, redistributable as part of an Isaac Sim / Omniverse Kit extension | Headers fetched at build time via NVIDIA packman; the SDK is not vendored in this repository. |
| NVIDIA OmniGraph | NVIDIA proprietary, same redistribution terms as Carbonite | Linked at extension load time via Kit's `RTLD_GLOBAL` symbol scope. |
| NVIDIA USD (Pixar fork) | Modified Apache-2.0 (Pixar Animation Studios) | Used via OmniGraph node interfaces; the plugin does not link `libusd_*` directly. |
| NVIDIA Replicator / RTX sensor OG nodes | Apache-2.0 | This extension wires upstream OG node outputs; Replicator source is not included here. |
| libcudart | NVIDIA proprietary, redistributable per the CUDA EULA | Required for CUDA-backed Isaac Sim sensor annotators; loaded by Isaac Sim, not by this extension directly. |

## Generation

The Rust dependency table was hand-curated from the workspace `Cargo.toml` and per-crate manifests.

Once `cargo-about` is installed and an `about.toml` configuration file is present, the table can be regenerated via:

```
cargo about generate -c about.toml > docs/THIRD_PARTY_LICENSES.md
```
