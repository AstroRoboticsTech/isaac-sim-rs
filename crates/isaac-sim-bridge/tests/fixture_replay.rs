//! Replays captured annotator output fixtures through the bridge's
//! dispatch chain to assert the C++→Rust shape still matches what
//! NVIDIA's RTX annotators emit. See `tests/fixtures/README.md`.
//!
//! Catches the `d7a102a`-class drift: a future Isaac Sim release
//! changing an annotator's output layout would break our publish
//! nodes silently at runtime; this test fails at cargo test time
//! instead, before anyone tries a kit launch.
//!
//! When no fixtures are present (CI without captured data), the test
//! is a no-op pass. The static schema validator in `ogn_schema.rs`
//! covers the .ogn side regardless.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use isaac_sim_bridge::{
    register_lidar_pointcloud_consumer, LidarPointCloudMeta,
};
use serde_json::Value;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(2)
        .expect("manifest dir has at least two ancestors")
        .to_path_buf()
}

fn list_fixtures(prefix: &str) -> Vec<(PathBuf, PathBuf)> {
    let dir = workspace_root().join("tests/fixtures");
    if !dir.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("read fixtures dir") {
        let path = entry.expect("entry").path();
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        if !stem.starts_with(prefix) {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("bin") {
            continue;
        }
        let meta = path.with_extension("meta.json");
        if meta.exists() {
            out.push((path, meta));
        }
    }
    out
}

#[test]
fn pointcloud_fixtures_replay_through_dispatch() {
    let fixtures = list_fixtures("lidar_pointcloud_");
    if fixtures.is_empty() {
        eprintln!("no lidar_pointcloud fixtures present; skipping replay");
        return;
    }

    // We register a sentinel consumer that ticks for each replayed
    // fixture; assert it fires once per fixture under our test source.
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_clone = Arc::clone(&hits);
    register_lidar_pointcloud_consumer(move |src, points, meta| {
        if src != "__fixture_replay__" {
            return;
        }
        // Spot-check the meta + slice agree.
        assert_eq!(
            points.len(),
            (meta.num_points as usize) * 3,
            "meta.num_points * 3 != points.len()"
        );
        hits_clone.fetch_add(1, Ordering::SeqCst);
    });

    for (bin, meta_path) in &fixtures {
        let raw = fs::read(bin).expect("read bin");
        let meta_json: Value =
            serde_json::from_slice(&fs::read(meta_path).expect("read meta")).expect("parse meta");

        let stride = meta_json
            .get("byte_stride")
            .and_then(Value::as_u64)
            .unwrap_or(4) as usize;
        let channels = meta_json
            .get("channels")
            .and_then(Value::as_u64)
            .unwrap_or(3) as usize;
        assert_eq!(stride, 4, "fixture stride must be sizeof(f32)");
        assert_eq!(channels, 3, "fixture channels must be 3 (XYZ)");

        let n_floats = raw.len() / 4;
        assert_eq!(n_floats % 3, 0, "fixture {bin:?} not a multiple of 3 floats");
        let n_points = n_floats / 3;

        // Reinterpret bytes as &[f32]. Safe because stride=4 and
        // bytemuck handles the alignment check.
        let points: &[f32] = bytemuck::cast_slice(&raw[..n_floats * 4]);
        let pc_meta = LidarPointCloudMeta {
            num_points: n_points as i32,
            width: n_points as i32,
            height: 1,
        };

        // Replay through forward_lidar_pointcloud. We don't have direct
        // access to the FFI fn from the test crate, so trigger the
        // dispatch via the public consumer-side API: register, dispatch
        // synthetic data, verify the shape line up.
        isaac_sim_bridge::dispatch_lidar_pointcloud("__fixture_replay__", points, &pc_meta);
    }

    assert_eq!(
        hits.load(Ordering::SeqCst),
        fixtures.len(),
        "expected one consumer hit per fixture",
    );
}

#[test]
fn fixture_meta_json_well_formed() {
    let fixtures = list_fixtures("lidar_");
    for (_bin, meta_path) in fixtures {
        let v: Value = serde_json::from_slice(&fs::read(&meta_path).expect("read"))
            .expect("parse meta json");
        assert!(v.get("annotator").is_some(), "{meta_path:?}: missing annotator");
        assert!(
            v.get("isaac_version").is_some(),
            "{meta_path:?}: missing isaac_version"
        );
        assert!(v.get("dtype").is_some(), "{meta_path:?}: missing dtype");
    }
}
