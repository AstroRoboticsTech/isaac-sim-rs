# isaac-sim-arrow

[![Crates.io](https://img.shields.io/crates/v/isaac-sim-arrow.svg)](https://crates.io/crates/isaac-sim-arrow)
[![Documentation](https://docs.rs/isaac-sim-arrow/badge.svg)](https://docs.rs/isaac-sim-arrow)
[![License](https://img.shields.io/badge/license-MPL--2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)

Pure-Rust Apache Arrow schema and decoders for every sensor and actuator channel exposed by the Isaac Sim bridge. Takes raw buffers from the `cxx::bridge` layer and produces `RecordBatch` values that any downstream — dora-rs node, rerun viewer, file logger, or custom bus — can consume without depending on Isaac Sim or a C++ toolchain.

## Usage

```toml
[dependencies]
isaac-sim-arrow = "0.1"
```

```rust
use isaac_sim_arrow::lidar::flatscan::{LidarFlatScan, to_record_batch, from_struct_array};
use arrow::array::StructArray;

let depths = [0.5_f32, 1.2, 2.7, 3.0];
let intensities = [10_u8, 50, 200, 100];

let scan = LidarFlatScan {
    depths: &depths,
    intensities: &intensities,
    horizontal_fov: 270.0,
    horizontal_resolution: 0.25,
    azimuth_min: -135.0,
    azimuth_max: 135.0,
    depth_min: 0.1,
    depth_max: 30.0,
    num_rows: 1,
    num_cols: 4,
    rotation_rate: 10.0,
};

let batch = to_record_batch(&scan)?;
// round-trip back to native struct
let owned = from_struct_array(&StructArray::from(batch))?;
assert_eq!(owned.depths, depths);
# Ok::<(), arrow::error::ArrowError>(())
```

Part of the [`isaac-sim-rs`](https://github.com/AstroRoboticsTech/isaac-sim-rs) SDK. The facade crate re-exports this as the `arrow` namespace when the `arrow` feature is enabled (the default).

## Compatibility

| isaac-sim-arrow | arrow major | Rust MSRV |
| --------------- | ----------- | --------- |
| 0.1             | 54          | 1.85      |

`arrow` is a public-surface dep: every public function returns or accepts an `arrow::array::*` or `arrow::record_batch::RecordBatch`. Pin the same `arrow` major as this crate.

Apache Arrow 54 is workspace-pinned to match the version dora-node-api 0.5 requires. Consumers must use the same Arrow major to share `RecordBatch` values across crate boundaries without copy.

## License notice for re-publishers

MPL-2.0 is per-file copyleft. If you bundle this crate's binary into your own crate or extension, retain the SPDX header on every source file you include. The full license text is in [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE) at the repo root.

## License

MPL-2.0 (see [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE)).
