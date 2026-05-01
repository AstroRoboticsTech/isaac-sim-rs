#!/usr/bin/env bash
# Tier-(c) Kit + dora E2E smoke test.
#
# Spins up the nova-carter-dora dataflow, waits for the receiver node to
# emit at least one log line per channel, and tears everything down.
#
# Usage:
#   ./scripts/run_kit_smoke_dora.sh                 # 90 s timeout default
#   KIT_SMOKE_TIMEOUT=120 ./scripts/run_kit_smoke_dora.sh
#
# Exit codes:
#   0  — all 8 channel log lines observed before timeout
#   1  — timeout without full coverage
#   2  — FATAL / panic / OmniGraphError in log
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

: "${ISAAC_SIM:?ISAAC_SIM env var not set}"
export ISAAC_SIM
ISAAC_SIM_RS="${ISAAC_SIM_RS:-$(pwd)}"
export ISAAC_SIM_RS

TIMEOUT="${KIT_SMOKE_TIMEOUT:-90}"
LOG_FILE="${KIT_SMOKE_DORA_LOG:-/tmp/isaac-rs-kit-smoke-dora.log}"

DORA="${DORA_BIN:-}"
if [ -z "$DORA" ]; then
    if command -v dora >/dev/null 2>&1; then
        DORA="dora"
    elif [ -x "${HOME}/.cargo/bin/dora" ]; then
        DORA="${HOME}/.cargo/bin/dora"
    else
        echo "[kit-smoke-dora] dora-cli not found on PATH or ~/.cargo/bin/dora" >&2
        exit 1
    fi
fi

REQUIRED_CHANNELS=(
    lidar_flatscan
    lidar_pointcloud
    camera_rgb
    camera_depth
    camera_info
    imu
    odometry
    cmd_vel_observed
)

teardown() {
    echo "[kit-smoke-dora] tearing down"
    "$DORA" destroy 2>/dev/null || true
    pkill -9 -f "kit/kit" 2>/dev/null || true
    pkill -9 -f "example-nova-carter-dora" 2>/dev/null || true
}
trap teardown EXIT

rm -f "$LOG_FILE"

echo "[kit-smoke-dora] building dataflow (timeout=${TIMEOUT}s log=${LOG_FILE})"
"$DORA" build examples/nova-carter-dora/dataflow.yml >>"$LOG_FILE" 2>&1

echo "[kit-smoke-dora] starting dataflow"
"$DORA" up >>"$LOG_FILE" 2>&1
RUN_ID=$("$DORA" start examples/nova-carter-dora/dataflow.yml --detach 2>>"$LOG_FILE")
echo "[kit-smoke-dora] run-id: ${RUN_ID}"

DEADLINE=$(( $(date +%s) + TIMEOUT ))

check_all_channels() {
    local log="$1"
    for ch in "${REQUIRED_CHANNELS[@]}"; do
        grep -q "\[receiver\] ${ch}:" "$log" 2>/dev/null || return 1
    done
    return 0
}

"$DORA" logs "$RUN_ID" receiver >>"$LOG_FILE" 2>&1 &

while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    if grep -qE "OmniGraphError|panic|FATAL" "$LOG_FILE" 2>/dev/null; then
        echo "[kit-smoke-dora] FATAL signal detected:"
        grep -E "OmniGraphError|panic|FATAL" "$LOG_FILE" | head -5
        exit 2
    fi
    if check_all_channels "$LOG_FILE"; then
        echo "[kit-smoke-dora] all channels confirmed:"
        for ch in "${REQUIRED_CHANNELS[@]}"; do
            grep "\[receiver\] ${ch}:" "$LOG_FILE" | tail -1
        done
        echo "[kit-smoke-dora] PASS"
        exit 0
    fi
    sleep 2
done

echo "[kit-smoke-dora] TIMEOUT after ${TIMEOUT}s — channels not yet seen:"
for ch in "${REQUIRED_CHANNELS[@]}"; do
    grep -q "\[receiver\] ${ch}:" "$LOG_FILE" 2>/dev/null \
        && echo "  [ok]  $ch" \
        || echo "  [MISSING] $ch"
done
echo "[kit-smoke-dora] last 10 log lines:"
tail -10 "$LOG_FILE" || true
exit 1
