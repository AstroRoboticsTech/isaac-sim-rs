# SPDX-License-Identifier: MPL-2.0
import? 'justfile.local'

bridge_dir := "cpp/omni.isaacsimrs.bridge"
build_dir  := bridge_dir + "/build"
bin_dir    := bridge_dir + "/bin"
dist_dir   := "dist"
ext_name   := "omni.isaacsimrs.bridge"
platform   := "linux-" + `uname -m`
cargo_jobs := env_var_or_default("CARGO_BUILD_JOBS", "")

default: check

check: fmt-check clippy test

fmt-check:
    cargo fmt --all -- --check

fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

build-rust:
    cargo build --workspace --lib

configure:
    @if [ -z "${{ "ISAAC_SIM_PATH" }}" ]; then echo "ISAAC_SIM_PATH not set" >&2; exit 1; fi
    cmake -S {{ bridge_dir }} -B {{ build_dir }}

build: configure
    cmake --build {{ build_dir }} {{ if cargo_jobs == "" { "" } else { "-j" + cargo_jobs } }}

link-smoke:
    ./scripts/run_cpp_link_smoke.sh

smoke: build
    ./scripts/run_kit_smoke.sh

smoke-tarball: package-extension
    ./scripts/run_kit_smoke_tarball.sh

package-extension: build
    cmake --install {{ build_dir }} --prefix {{ dist_dir }}
    @VERSION=$(grep '^version' {{ bridge_dir }}/config/extension.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/'); \
     TARBALL="{{ dist_dir }}/{{ ext_name }}-${VERSION}-{{ platform }}.tar.gz"; \
     tar -czf "$TARBALL" -C {{ dist_dir }} {{ ext_name }}/ && \
     sha256sum "$TARBALL" > "${TARBALL}.sha256" && \
     echo "tarball: $TARBALL" && \
     echo "sha256:  ${TARBALL}.sha256"
    @echo "--- ldd unresolved deps (none expected) ---"
    -ldd {{ dist_dir }}/{{ ext_name }}/bin/{{ platform }}/*.so | grep -v '=>' | grep -v 'linux-vdso\|/lib'

release-kit-tarball tag: package-extension
    @command -v gh >/dev/null || (echo "gh CLI required (https://cli.github.com)"; exit 1)
    @gh auth status >/dev/null 2>&1 || (echo "run 'gh auth login' first"; exit 1)
    @gh release view {{ tag }} >/dev/null 2>&1 || (echo "release {{ tag }} not found; tag the repo + push first"; exit 1)
    @VERSION=$(grep '^version' {{ bridge_dir }}/config/extension.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/'); \
     TARBALL="{{ dist_dir }}/{{ ext_name }}-${VERSION}-{{ platform }}.tar.gz"; \
     [ -f "$TARBALL" ] || (echo "tarball $TARBALL missing"; exit 1); \
     echo "uploading $TARBALL + sha256 to release {{ tag }}..."; \
     gh release upload {{ tag }} "$TARBALL" "${TARBALL}.sha256" --clobber; \
     echo "done: $(gh release view {{ tag }} --json url -q .url)"

verify-rpath:
    @echo "--- ldd resolution post-install ---"
    @cd {{ dist_dir }}/{{ ext_name }}/bin/{{ platform }} && \
     for so in *.so; do \
         echo "checking $so..."; \
         readelf -d "$so" | grep -E "RUNPATH|RPATH" || echo "  no RUNPATH/RPATH (relies on LD_LIBRARY_PATH)"; \
         ldd "$so" | grep "not found" && echo "  UNRESOLVED" && exit 1 || true; \
     done
    @echo "ok: all libs resolve"

clean:
    cargo clean
    rm -rf {{ build_dir }}
    rm -rf {{ bin_dir }}
