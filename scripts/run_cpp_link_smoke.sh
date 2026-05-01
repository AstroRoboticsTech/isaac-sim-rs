#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Tier-(b) link smoke: verifies the cxx::bridge ABI between C++ and
# libisaac_sim_bridge.so without Kit, USD, or a GPU. Builds the cdylib,
# configures the link_smoke cmake project, builds the smoke executable,
# and runs it. Non-zero exit on any step means the FFI surface drifted.
#
# Usage:
#   ./scripts/run_cpp_link_smoke.sh           # release profile (default)
#   CARGO_PROFILE=debug ./scripts/run_cpp_link_smoke.sh
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PROFILE="${CARGO_PROFILE:-release}"

echo "[link-smoke] cargo build -p isaac-sim-bridge --${PROFILE}"
if [[ "$PROFILE" == "release" ]]; then
    cargo build -p isaac-sim-bridge --release
else
    cargo build -p isaac-sim-bridge
fi

BUILD_DIR="tests/cpp/link_smoke/build"
echo "[link-smoke] cmake -S tests/cpp/link_smoke -B ${BUILD_DIR}"
CARGO_PROFILE="$PROFILE" cmake -S tests/cpp/link_smoke -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release

echo "[link-smoke] cmake --build ${BUILD_DIR}"
cmake --build "$BUILD_DIR" -- -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2)"

echo "[link-smoke] running ./${BUILD_DIR}/link_smoke"
"./${BUILD_DIR}/link_smoke"
