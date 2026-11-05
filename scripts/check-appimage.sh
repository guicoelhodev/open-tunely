#!/usr/bin/env bash
# Checks payload integrity and executable tools without requiring FUSE or a display.
set -euo pipefail
image=$(realpath "${1:?Usage: bash scripts/check-appimage.sh path/to/app.AppImage}")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
"$image" --appimage-extract > /dev/null
app="$work/squashfs-root"
tools="$app/usr/lib/Open Tunely/binaries"
for tool in yt-dlp deno ffmpeg ffprobe; do
  test -x "$tools/$tool"
done
test -s "$tools/deno-LICENSE.md"
test -s "$tools/ffmpeg-LICENSE.txt"
"$tools/yt-dlp" --version
"$tools/deno" --version
"$tools/ffmpeg" -version
"$tools/ffprobe" -version
test -d "$app/usr/lib/gstreamer-1.0"
if find "$app/usr/lib" -name libgstpython.so -print -quit | rg -q .; then
  echo "Unexpected Python GStreamer plugin (requires unbundled Python stdlib)" >&2
  exit 1
fi
test -x "$app/usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"
# Cover the playback pipeline and the MP3/AAC/Opus/Vorbis formats used by the app.
for plugin in debugutilsbad subenc id3demux audiofx playback audioconvert audioresample typefindfunctions autodetect soup mpg123 opus vorbis matroska isomp4; do
  test -f "$app/usr/lib/gstreamer-1.0/libgst$plugin.so" || { echo "Missing GStreamer plugin: $plugin" >&2; exit 1; }
done
# AAC may be decoded by either libav or FAAD, depending on the build distribution.
test -f "$app/usr/lib/gstreamer-1.0/libgstlibav.so" || test -f "$app/usr/lib/gstreamer-1.0/libgstfaad.so"
echo 'AppImage payload checks passed (GUI/playback still require a desktop smoke test).'
