fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++17")
        .compile("isaac_sim_bridge_cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
