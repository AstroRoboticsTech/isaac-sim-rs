// SPDX-License-Identifier: MPL-2.0
fn main() {
    if std::env::var_os("CARGO_FEATURE_CDYLIB").is_none() {
        return;
    }
    cxx_build::bridge("src/lib.rs")
        .std("c++17")
        .warnings(true)
        .compile("isaac_sim_bridge_cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/channel.rs");
    println!("cargo:rerun-if-changed=src/lifecycle.rs");
    println!("cargo:rerun-if-changed=src/demo.rs");
    println!("cargo:rerun-if-changed=src/lidar/mod.rs");
    println!("cargo:rerun-if-changed=src/lidar/flatscan.rs");
    println!("cargo:rerun-if-changed=src/lidar/pointcloud.rs");
    println!("cargo:rerun-if-changed=src/camera/mod.rs");
    println!("cargo:rerun-if-changed=src/camera/rgb.rs");
    println!("cargo:rerun-if-changed=src/camera/depth.rs");
}
