use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(carb_bindings)");
    println!("cargo::rerun-if-env-changed=CARB_INCLUDE_DIR");

    let Ok(include_dir) = env::var("CARB_INCLUDE_DIR") else {
        println!(
            "cargo::warning=carb-sys: CARB_INCLUDE_DIR not set; skipping bindgen. \
             Set CARB_INCLUDE_DIR=<isaac-sim>/kit/dev/include to generate bindings."
        );
        return;
    };

    let header = format!("{include_dir}/carb/Framework.h");
    println!("cargo::rerun-if-changed={header}");

    let bindings = bindgen::Builder::default()
        .header(&header)
        .clang_arg(format!("-I{include_dir}"))
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .allowlist_item("carb.*")
        .allowlist_item("Carb.*")
        .allowlist_item("CARB_.*")
        .opaque_type("std::.*")
        .layout_tests(false)
        .generate()
        .expect("bindgen failed to generate Carb bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings.rs");

    println!("cargo::rustc-cfg=carb_bindings");
}
