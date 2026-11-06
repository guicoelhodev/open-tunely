#!/usr/bin/env bash
# Apply after linuxdeploy and before appimagetool.
set -euo pipefail
app=$(realpath "${1:?Usage: bash scripts/fix-appimage-runtime.sh path/to/AppDir}")
# Python bindings are unnecessary for HTML audio and require an unbundled stdlib.
find "$app/usr/lib" -name libgstpython.so -delete
binary="$app/usr/bin/open-tunely"
# Restore the native executable if an earlier launcher wrapped it.
if [ -f "$binary.bin" ]; then
  mv -f "$binary.bin" "$binary"
fi
# Run hooks before setting bundled library paths. A host shell started after
# that point can load incompatible readline, even when /bin/sh is Bash.
cat > "$app/AppRun" <<'APPRUN'
#!/bin/sh
set -e
app="$(dirname "$(readlink -f "$0")")"
export APPDIR="$app"
for hook in "$app"/apprun-hooks/*.sh; do
  [ -f "$hook" ] || continue
  . "$hook"
done
export LD_LIBRARY_PATH="$app/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$app/usr/bin:$PATH"
export WEBKIT_EXEC_PATH="$app/usr/lib/webkit2gtk-4.1"
export WEBKIT_INJECTED_BUNDLE_PATH="$app/usr/lib/webkit2gtk-4.1/injected-bundle"
unset PYTHONHOME PYTHONPATH
# linuxdeploy rewrites WebKit helper paths relative to usr.
cd "$app/usr"
exec "$app/usr/bin/open-tunely" "$@"
APPRUN
chmod +x "$app/AppRun"
rm -f "$app/AppRun.wrapped"
# Audio playback does not need the Vulkan video plugin. Its discovery probes
# graphics drivers and can emit RADV diagnostics even when no video is played.
find "$app/usr/lib" -name libgstvulkan.so -delete
