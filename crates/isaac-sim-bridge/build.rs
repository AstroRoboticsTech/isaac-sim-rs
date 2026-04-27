fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++17")
        .warnings(true)
        .compile("isaac_sim_bridge_cxx");
    // The cxx::bridge mod is in lib.rs, but extern "Rust" implementations
    // live in submodules; touching any of those needs a rebuild of the
    // generated shim, not just the rlib.
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/channel.rs");
    println!("cargo:rerun-if-changed=src/lifecycle.rs");
    println!("cargo:rerun-if-changed=src/demo.rs");
    println!("cargo:rerun-if-changed=src/lidar/mod.rs");
    println!("cargo:rerun-if-changed=src/lidar/flatscan.rs");
    println!("cargo:rerun-if-changed=src/lidar/pointcloud.rs");
}
