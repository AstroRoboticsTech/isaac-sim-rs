#!/usr/bin/env bash
# Tier-(c) Kit-launched smoke test for nightly cron.
#
# Boots Kit headlessly via the nova-carter example launcher, watches
# for rerun batcher backpressure warnings (proxy for "consumers fired
# at least once"), and exits 0 on success / non-zero on timeout or
# crash. Wire as a nightly cron writing JUnit-XML or a one-line status
# file.
#
# Usage on the workstation:
#   ./scripts/run_kit_smoke.sh                 # 90s timeout default
#   KIT_SMOKE_TIMEOUT=120 ./scripts/run_kit_smoke.sh
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

: "${ISAAC_SIM:?ISAAC_SIM env var not set}"
export ISAAC_SIM
ISAAC_SIM_RS="${ISAAC_SIM_RS:-$(pwd)}"
export ISAAC_SIM_RS
TIMEOUT="${KIT_SMOKE_TIMEOUT:-90}"
LOG_FILE="${KIT_SMOKE_LOG:-/tmp/isaac-rs-kit-smoke.log}"

pkill -9 -f kit/kit 2>/dev/null || true
sleep 1
rm -f "$LOG_FILE"

echo "[kit-smoke] launching Kit (timeout=${TIMEOUT}s log=${LOG_FILE})"
setsid bash -c "exec ${ISAAC_SIM_RS}/examples/nova-carter/launch.sh" \
    >"$LOG_FILE" 2>&1 < /dev/null &

DEADLINE=$(( $(date +%s) + TIMEOUT ))
while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    if grep -q "re_quota_channel" "$LOG_FILE" 2>/dev/null; then
        echo "[kit-smoke] dispatch confirmed via rerun batcher activity"
        pkill -9 -f kit/kit 2>/dev/null || true
        echo "[kit-smoke] PASS"
        exit 0
    fi
    if grep -qE "OmniGraphError|panic|FATAL" "$LOG_FILE" 2>/dev/null; then
        echo "[kit-smoke] FATAL signal detected:"
        grep -E "OmniGraphError|panic|FATAL" "$LOG_FILE" | head -5
        pkill -9 -f kit/kit 2>/dev/null || true
        exit 2
    fi
    sleep 2
done

echo "[kit-smoke] TIMEOUT after ${TIMEOUT}s without dispatch"
echo "[kit-smoke] last 10 log lines:"
tail -10 "$LOG_FILE" || true
pkill -9 -f kit/kit 2>/dev/null || true
exit 1
