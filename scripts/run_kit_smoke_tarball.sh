#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Tarball smoke test: extracts the just-built kit extension tarball into
# a temp ext folder, launches Kit headlessly with that folder on its
# extension search path, and verifies the extension loads cleanly.
#
# Use this before tagging a release to confirm the GitHub Releases
# tarball is a working drop-in install.
#
# Usage:
#   ./scripts/run_kit_smoke_tarball.sh                # 60s timeout default
#   KIT_SMOKE_TIMEOUT=120 ./scripts/run_kit_smoke_tarball.sh
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

: "${ISAAC_SIM:?ISAAC_SIM env var not set}"
ISAAC_SIM_RS="${ISAAC_SIM_RS:-$(pwd)}"
TIMEOUT="${KIT_SMOKE_TIMEOUT:-60}"
LOG_FILE="${KIT_SMOKE_LOG:-/tmp/isaac-rs-tarball-smoke.log}"
EXTRACT_DIR="${KIT_SMOKE_EXTRACT_DIR:-/tmp/isaac-rs-tarball-smoke-ext}"

PLATFORM="linux-$(uname -m)"
TARBALL=$(ls "${ISAAC_SIM_RS}/dist"/omni.isaacsimrs.bridge-*-"${PLATFORM}".tar.gz 2>/dev/null | head -1 || true)
if [ -z "${TARBALL:-}" ] || [ ! -f "$TARBALL" ]; then
    echo "[tarball-smoke] no tarball in ${ISAAC_SIM_RS}/dist/ — run 'just package-extension' first"
    exit 1
fi
echo "[tarball-smoke] tarball: $TARBALL ($(du -h "$TARBALL" | cut -f1))"

rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$TARBALL" -C "$EXTRACT_DIR"
echo "[tarball-smoke] extracted to $EXTRACT_DIR"
echo "[tarball-smoke] extension contents:"
find "$EXTRACT_DIR/omni.isaacsimrs.bridge" -maxdepth 2 -type f | sed 's/^/  /'

pkill -9 -f kit/kit 2>/dev/null || true
sleep 1
rm -f "$LOG_FILE"

echo "[tarball-smoke] launching Kit (timeout=${TIMEOUT}s log=${LOG_FILE})"
setsid stdbuf -oL -eL \
    "${ISAAC_SIM}/kit/kit" \
    "${ISAAC_SIM}/apps/isaacsim.exp.full.kit" \
    --no-window \
    --no-ros-env \
    --ext-folder "$EXTRACT_DIR" \
    --enable omni.isaacsimrs.bridge \
    >"$LOG_FILE" 2>&1 < /dev/null &

DEADLINE=$(( $(date +%s) + TIMEOUT ))
while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    if grep -qE "\[ext: omni\.isaacsimrs\.bridge-[0-9].*\] startup" "$LOG_FILE" 2>/dev/null; then
        echo "[tarball-smoke] extension startup line detected:"
        grep -E "\[ext: omni\.isaacsimrs\.bridge|isaacsimrs|OgnPublish|OgnApply" "$LOG_FILE" | head -20
        pkill -9 -f kit/kit 2>/dev/null || true
        echo "[tarball-smoke] PASS"
        exit 0
    fi
    if grep -qE "Failed to load extension.*omni\.isaacsimrs|extension not found.*omni\.isaacsimrs|\[Error\].*omni\.isaacsimrs\.bridge|panic|FATAL|Segmentation fault" "$LOG_FILE" 2>/dev/null; then
        echo "[tarball-smoke] failure signal detected:"
        grep -E "Failed to load extension|extension not found|\[Error\].*omni\.isaacsimrs|panic|FATAL|Segmentation fault" "$LOG_FILE" | head -10
        pkill -9 -f kit/kit 2>/dev/null || true
        exit 2
    fi
    sleep 1
done

echo "[tarball-smoke] TIMEOUT after ${TIMEOUT}s"
echo "[tarball-smoke] last 30 log lines:"
tail -30 "$LOG_FILE" || true
pkill -9 -f kit/kit 2>/dev/null || true
exit 1
