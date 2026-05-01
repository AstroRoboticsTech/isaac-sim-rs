// SPDX-License-Identifier: MPL-2.0
//! Raw FFI bindings to the NVIDIA Omniverse Carbonite (Carb) SDK.
//!
//! Bindings are generated at build time from `<isaac-sim>/kit/dev/include/`.
//! Set `CARB_INCLUDE_DIR` to enable; otherwise this crate is empty.
#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]

#[cfg(carb_bindings)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
