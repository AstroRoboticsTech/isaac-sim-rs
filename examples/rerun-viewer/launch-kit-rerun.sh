#!/bin/bash
set -euo pipefail

: "${ISAAC_SIM:?ISAAC_SIM env var not set; export it before running this script}"
: "${ISAAC_SIM_RS:?ISAAC_SIM_RS env var not set; export it before running this script}"

export ISAAC_SIM_RS_RERUN_RUNNER="${ISAAC_SIM_RS}/cpp/omni.isaacsimrs.bridge/bin/libexample_rerun_viewer.so"
export ISAAC_SIM_RS_RERUN_GRPC_ADDR="${ISAAC_SIM_RS_RERUN_GRPC_ADDR:-127.0.0.1:9876}"

exec "${ISAAC_SIM}/kit/kit" \
    "${ISAAC_SIM}/apps/isaacsim.exp.full.kit" \
    --no-window \
    --no-ros-env \
    --ext-folder "${ISAAC_SIM_RS}/cpp" \
    --enable omni.isaacsimrs.bridge \
    --enable isaacsim.sensors.rtx \
    --exec "${ISAAC_SIM_RS}/examples/rerun-viewer/drive.py"
