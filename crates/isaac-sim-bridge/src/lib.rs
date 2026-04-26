use std::sync::Once;

static INIT: Once = Once::new();

#[no_mangle]
pub extern "C" fn isaac_sim_rs_init() {
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .try_init();
    });
    log::info!("[isaac-sim-rs] hello from Rust isaac_sim_rs_init");
}
