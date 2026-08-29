#!/usr/bin/env bash
# Bring up a headless X display so the eframe/egui GUI can run on a VM with no
# GPU or physical screen. Rendering falls back to Mesa software drivers
# (llvmpipe for GL, lavapipe for Vulkan/wgpu). Idempotent: safe to re-run.
set -euo pipefail

DISPLAY_NUM=":99"

if ! pgrep -x Xvfb >/dev/null 2>&1; then
    nohup Xvfb "${DISPLAY_NUM}" -screen 0 1280x800x24 -ac +extension GLX +render -noreset \
        >/tmp/xvfb.log 2>&1 &
fi

# Wait for the display to accept connections before returning.
for _ in $(seq 1 40); do
    if DISPLAY="${DISPLAY_NUM}" xdpyinfo >/dev/null 2>&1; then
        echo "Xvfb ready on ${DISPLAY_NUM}"
        exit 0
    fi
    sleep 0.25
done

echo "warning: Xvfb did not become ready on ${DISPLAY_NUM}" >&2
exit 0
