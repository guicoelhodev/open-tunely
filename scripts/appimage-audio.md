# Audio AppImage optimization

After linuxdeploy populates an AppDir, and before appimagetool compresses it:

```sh
bash scripts/fix-appimage-runtime.sh /path/to/AppDir
python3 scripts/prune-appimage.py /path/to/AppDir
python3 scripts/check-appimage-audio.py /path/to/AppDir
node scripts/check-media-tools.mjs '/path/to/AppDir/usr/lib/Open Tunely/binaries'
```

The pruning script retains audio plugins and removes only bundled libraries that
are dependencies of removed plugins and are not dependencies of retained ELF
executables/modules. Dynamically loaded graphics libraries are preserved.
WebKitGTK, JavaScriptCore, Deno and the managed download/conversion tools remain.

The playback check requires Python, openssl, host FFmpeg (fixture generation),
gst-launch-1.0 matching the bundled GStreamer version, and local socket access.
It uses the AppDir plugin/scanner/library paths and a fresh plugin registry.
It decodes MP3, M4A/AAC, WebM/Opus, Ogg/Vorbis and FLAC from files and local
HTTP/HTTPS servers. TLS certificate checking is disabled only on the test source
for its self-signed fixture; application TLS settings are untouched.
Output goes to a fake audio sink, so audible playback and end-to-end YouTube
preview still require desktop testing. After compression run check-appimage.sh
and launch the resulting image under a real display or xvfb-run for a smoke test.
