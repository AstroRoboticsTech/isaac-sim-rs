import? 'justfile.local'

bridge_dir := "cpp/omni.isaacsimrs.bridge"
build_dir  := bridge_dir + "/build"
bin_dir    := bridge_dir + "/bin"
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

clean:
    cargo clean
    rm -rf {{ build_dir }}
    rm -rf {{ bin_dir }}
