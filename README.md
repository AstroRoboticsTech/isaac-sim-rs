# isaac-sim-rs

Unofficial Rust SDK for [NVIDIA Isaac Sim](https://developer.nvidia.com/isaac/sim) & [Omniverse](https://developer.nvidia.com/omniverse).

Maintained by [Astro Robotics](https://github.com/AstroRoboticsTech).

## Status

Early development. Not yet ready for production use. The current focus is direct C++ FFI bindings to Isaac Sim's RTX sensor pipeline, OmniGraph runtime, Carb plugin SDK, and USD stage — designed to plug into [`dora-rs`](https://github.com/dora-rs/dora) dataflows without a Python or ROS bridge in the hot path.

## License

Licensed under the [Mozilla Public License 2.0](LICENSE).

You may use this SDK in any project — commercial, proprietary, or open source. If you modify source files in this repository, your changes to those files must be released under MPL-2.0. See the [LICENSE](LICENSE) file for the full terms.

## Prior art

- [`AndrejOrsula/omniverse_rs`](https://github.com/AndrejOrsula/omniverse_rs) — autocxx-based Omniverse interface (dormant since 2024)
- [`AndrejOrsula/isaac_sim_rs`](https://github.com/AndrejOrsula/isaac_sim_rs) — Rust interface for Isaac Sim (dormant since 2024)
- [`AndrejOrsula/pxr_rs`](https://github.com/AndrejOrsula/pxr_rs) — autocxx-based OpenUSD bindings
