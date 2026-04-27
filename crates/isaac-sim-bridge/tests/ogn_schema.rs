//! Static schema validation for the bridge's OGN node files.
//!
//! Catches the d7a102a bug class — schema authoring mistakes that compile
//! and load but silently fail at runtime. Specifically:
//!
//! - `token`-typed inputs with `default: ""` are silently stripped from
//!   the registered Database.h. Use `string` (with default `""`) instead.
//! - Every sensor publish node must declare `inputs:sourceId` (the
//!   per-source routing key adapters filter on).
//! - Every node that accepts a GPU-resident pointer (`dataPtr`,
//!   `*Ptr`-suffixed inputs) must declare `inputs:cudaDeviceIndex`
//!   alongside, otherwise the C++ side has no way to know whether to
//!   `cudaMemcpyDeviceToHost`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(2)
        .expect("manifest dir has at least two ancestors")
        .to_path_buf()
}

fn load_ogn_files() -> Vec<(String, Value)> {
    let nodes_dir = workspace_root().join("cpp/omni.isaacsimrs.bridge/nodes");
    let mut out = Vec::new();
    for entry in fs::read_dir(&nodes_dir).expect("read nodes dir") {
        let path = entry.expect("read entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("ogn") {
            let raw = fs::read_to_string(&path).expect("read ogn file");
            let v: Value = serde_json::from_str(&raw).expect("ogn is valid json");
            out.push((file_name(&path), v));
        }
    }
    assert!(!out.is_empty(), "no .ogn files found under {nodes_dir:?}");
    out
}

fn file_name(p: &Path) -> String {
    p.file_name().unwrap().to_string_lossy().into_owned()
}

fn each_node(ogn: &Value) -> impl Iterator<Item = (&String, &Value)> {
    ogn.as_object()
        .expect("ogn root is object")
        .iter()
        .filter(|(k, _)| !k.starts_with('$'))
}

fn each_input(node: &Value) -> impl Iterator<Item = (&String, &Value)> {
    node.get("inputs")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.iter())
}

fn input_type(spec: &Value) -> &str {
    spec.get("type").and_then(Value::as_str).unwrap_or("")
}

#[test]
fn no_token_input_has_empty_default() {
    // `token` + `default: ""` is the silent-default-drop trap. The OGN
    // codegen builds Database.h without the default; the OG push
    // evaluator then refuses to schedule compute() on the publish node.
    for (file, ogn) in load_ogn_files() {
        for (node_name, node) in each_node(&ogn) {
            for (input_name, spec) in each_input(node) {
                if input_type(spec) == "token" {
                    let default = spec.get("default");
                    let is_empty_string = default.and_then(Value::as_str) == Some("");
                    assert!(
                        !is_empty_string,
                        "{file}/{node_name}: input '{input_name}' is type=token with default=\"\"; \
                         OGN strips empty defaults from token type. Use type=string instead.",
                    );
                }
            }
        }
    }
}

#[test]
fn every_publish_node_declares_source_id() {
    // Every sensor publish node carries a sourceId so adapters can
    // filter dispatches per-source. Missing it means a multi-sensor
    // dataflow can't tell two LiDARs apart.
    for (file, ogn) in load_ogn_files() {
        for (node_name, node) in each_node(&ogn) {
            if !node_name.starts_with("Publish") {
                continue;
            }
            let has_source_id = each_input(node).any(|(k, _)| k == "sourceId");
            assert!(
                has_source_id,
                "{file}/{node_name}: publish node missing inputs:sourceId",
            );
        }
    }
}

#[test]
fn data_ptr_inputs_declare_cuda_device_index() {
    // Any node that accepts a raw pointer input (dataPtr, *Ptr-suffixed)
    // must declare cudaDeviceIndex so the C++ compute() can decide
    // whether to cudaMemcpyDeviceToHost. Forgetting it on a new sensor
    // means the host-vs-GPU branch can't be made.
    for (file, ogn) in load_ogn_files() {
        for (node_name, node) in each_node(&ogn) {
            let has_ptr_input = each_input(node).any(|(k, _)| k == "dataPtr" || k.ends_with("Ptr"));
            if !has_ptr_input {
                continue;
            }
            let has_cuda_idx = each_input(node).any(|(k, _)| k == "cudaDeviceIndex");
            assert!(
                has_cuda_idx,
                "{file}/{node_name}: declares pointer input(s) but no inputs:cudaDeviceIndex sibling",
            );
        }
    }
}

#[test]
fn source_id_is_string_type_not_token() {
    // Defensive: even a non-empty default token would still be brittle
    // (token interning, default value semantics on path-typed strings).
    // String with default "" is the only correct shape.
    for (file, ogn) in load_ogn_files() {
        for (node_name, node) in each_node(&ogn) {
            for (input_name, spec) in each_input(node) {
                if input_name != "sourceId" {
                    continue;
                }
                let ty = input_type(spec);
                assert_eq!(
                    ty, "string",
                    "{file}/{node_name}: inputs:sourceId is type='{ty}'; must be type='string'",
                );
            }
        }
    }
}
