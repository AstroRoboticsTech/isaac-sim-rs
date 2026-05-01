# Compatibility matrix

| omni.isaacsimrs.bridge | Isaac Sim | Kit Kernel | USD (packman)                      | Status   |
| ---------------------- | --------- | ---------- | ---------------------------------- | -------- |
| 0.1.0                  | 5.1       | >=106.0    | 0.24.05.kit.7-gl.16400+05f48f24    | Verified |

The extension is currently Linux x86_64 only. Windows and macOS are accepted future work; the per-platform layout (`bin/${platform}/`) is in place but no binaries are produced.

The `[dependencies]` version floors in `config/extension.toml` are conservative lower bounds (`>=2.0.0` for `omni.graph.core`, `>=2.0.0` for `omni.kit.app`, `>=5.1.0` for `isaacsim.core.experimental`). The `omni.graph.core` floor was verified against the local Isaac Sim 5.1 install (`2.184.5`); the others are conservative and should be updated once a wider version sweep is run.

Run `just verify-rpath` after `just package-extension` before publishing a tarball to confirm all bundled shared libraries resolve correctly.

## Older Isaac Sim versions

Not supported. The OGN codegen toolchain pinned in `cmake/Packman.cmake` is Kit 106 / Isaac Sim 5.1 specific. A 4.x-compat fork would need to track its own packman version.
