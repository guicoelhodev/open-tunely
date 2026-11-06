#!/usr/bin/env bash
# Detect early loader/launcher failures; this does not validate interactive playback.
set -euo pipefail
image=$(realpath "${1:?Usage: bash scripts/check-appimage-startup.sh path/to/app.AppImage}")
log=$(mktemp)
trap 'rm -f "$log"' EXIT
set +e
timeout --kill-after=5s 15s dbus-run-session -- xvfb-run -a "$image" --appimage-extract-and-run > "$log" 2>&1
status=$?
set -e
cat "$log"
if [ "$status" -ne 124 ]; then
  echo "AppImage exited before the startup smoke-test deadline (status $status)." >&2
  exit 1
fi
if rg -qi 'symbol lookup error|error while loading shared libraries|segmentation fault' "$log"; then
  echo 'AppImage startup reported a loader failure.' >&2
  exit 1
fi
echo 'AppImage remained running throughout the startup smoke test.'
