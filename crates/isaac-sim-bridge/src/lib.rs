use std::sync::Once;

static INIT: Once = Once::new();

#[cxx::bridge(namespace = "isaacsimrs")]
mod ffi {
    extern "Rust" {
        fn init();
        fn double_value(x: i32) -> i32;
    }
}

fn init() {
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .try_init();
        log::info!("[isaac-sim-rs] init: env_logger up");
    });
}

fn double_value(x: i32) -> i32 {
    log::info!("[isaac-sim-rs] double_value({}) called from C++", x);
    x * 2
}
