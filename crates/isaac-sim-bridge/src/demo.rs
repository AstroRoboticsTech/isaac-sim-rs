// SPDX-License-Identifier: MPL-2.0
pub fn double_value(x: i32) -> i32 {
    log::info!("[isaac-sim-rs] double_value({}) called from C++", x);
    x * 2
}
