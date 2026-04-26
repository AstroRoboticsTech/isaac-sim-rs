#!/bin/bash
set -euo pipefail

: "${ISAAC_SIM:?ISAAC_SIM env var not set; export it before running 'dora start'}"
: "${ISAAC_SIM_RS:?ISAAC_SIM_RS env var not set; export it before running 'dora start'}"

export ISAAC_SIM_RS_DORA_RUNNER="${ISAAC_SIM_RS}/cpp/omni.isaacsimrs.bridge/bin/libisaac_sim_dora.so"
export ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT="${ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT:-lidar_flatscan}"

exec "${ISAAC_SIM}/kit/kit" \
    "${ISAAC_SIM}/apps/isaacsim.exp.base.kit" \
    --no-window \
    --no-ros-env \
    --ext-folder "${ISAAC_SIM_RS}/cpp" \
    --enable omni.isaacsimrs.bridge \
    --enable isaacsim.sensors.rtx \
    --exec "${ISAAC_SIM_RS}/examples/lidar-receiver/drive.py"
