#!/usr/bin/env bash
# Apply after linuxdeploy and before appimagetool.
set -euo pipefail
app=$(realpath "${1:?Usage: bash scripts/fix-appimage-runtime.sh path/to/AppDir}")
# Python bindings are unnecessary for HTML audio and require an unbundled stdlib.
find "$app/usr/lib" -name libgstpython.so -delete
binary="$app/usr/bin/open-tunely"
if [ ! -f "$binary.bin" ]; then
  mv "$binary" "$binary.bin"
fi
cat > "$binary" <<'LAUNCHER'
#!/usr/bin/env bash
# AppRun sets these for Python apps; this application uses standalone tools.
unset PYTHONHOME PYTHONPATH
exec "$(dirname "$(readlink -f "$0")")/open-tunely.bin" "$@"
LAUNCHER
chmod +x "$binary"
# The generic AppRun binary injects a Python environment before starting the app.
# Use a launcher tailored to this native application instead.
cat > "$app/AppRun.wrapped" <<'APPRUN'
#!/usr/bin/env bash
set -e
app="$(dirname "$(readlink -f "$0")")"
export APPDIR="$app"
export LD_LIBRARY_PATH="$app/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$app/usr/bin:$PATH"
export WEBKIT_EXEC_PATH="$app/usr/lib/webkit2gtk-4.1"
export WEBKIT_INJECTED_BUNDLE_PATH="$app/usr/lib/webkit2gtk-4.1/injected-bundle"
unset PYTHONHOME PYTHONPATH
# linuxdeploy rewrites WebKit helper paths relative to usr.
cd "$app/usr"
exec "$app/usr/bin/open-tunely" "$@"
APPRUN
chmod +x "$app/AppRun.wrapped"
# Audio playback does not need the Vulkan video plugin. Its discovery probes
# graphics drivers and can emit RADV diagnostics even when no video is played.
find "$app/usr/lib" -name libgstvulkan.so -delete
