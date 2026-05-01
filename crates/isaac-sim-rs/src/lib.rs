// SPDX-License-Identifier: MPL-2.0
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
//! Rust SDK for NVIDIA Isaac Sim.
//!
//! Adapter selection happens at compile time via cargo features:
//! - default: `arrow` only (pure Rust, no Isaac Sim required)
//! - `dora`:  bridge + dora pub/sub adapter
//! - `rerun`: bridge + rerun viewer adapter
//! - `full`:  both adapters in one cdylib
//!
//! The `bridge` feature enables the bridge rlib but NOT the cdylib C++ shim.
//! The C++ extension build passes `--features isaac-sim-bridge/cdylib` separately
//! so that a `cargo add isaac-sim-rs -F bridge` on a laptop without Isaac Sim
//! succeeds without requiring a C++ toolchain.
//!
//! See the workspace README for the full compatibility matrix.

/// Pure-Rust Arrow schema and decoders for every sensor and actuator channel.
#[cfg(feature = "arrow")]
#[cfg_attr(docsrs, doc(cfg(feature = "arrow")))]
pub use isaac_sim_arrow as arrow;

/// Bridge consumer registry, channel markers, producer registry, and `SourceFilter`.
#[cfg(feature = "bridge")]
#[cfg_attr(docsrs, doc(cfg(feature = "bridge")))]
pub use isaac_sim_bridge as bridge;

/// Dora-rs publisher and subscriber adapter; see [`crate::dora::subscribe`] for receiver-side decoders.
#[cfg(feature = "dora")]
#[cfg_attr(docsrs, doc(cfg(feature = "dora")))]
pub use isaac_sim_dora as dora;

/// Rerun viewer adapter; see [`crate::rerun::Viewer`] for the builder API.
#[cfg(feature = "rerun")]
#[cfg_attr(docsrs, doc(cfg(feature = "rerun")))]
pub use isaac_sim_rerun as rerun;
