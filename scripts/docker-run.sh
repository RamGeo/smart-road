#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

xhost +local:docker 2>/dev/null || true

args=(
  run --rm
  -e "DISPLAY=${DISPLAY:-:0}"
  -e SDL_AUDIODRIVER=pulse
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw
)

if [[ -e /mnt/wslg/PulseServer ]]; then
  args+=(
    -e PULSE_SERVER=unix:/mnt/wslg/PulseServer
    -v /mnt/wslg/PulseServer:/mnt/wslg/PulseServer:ro
  )
fi

exec docker "${args[@]}" smart-road:local
