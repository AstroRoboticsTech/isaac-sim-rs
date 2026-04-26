pub fn double_value(x: i32) -> i32 {
    log::info!("[isaac-sim-rs] double_value({}) called from C++", x);
    x * 2
}
