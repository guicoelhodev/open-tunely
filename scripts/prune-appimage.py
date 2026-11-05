#!/usr/bin/env python3
"""Keep audio GStreamer plugins and remove their unused ELF dependencies."""
import os
from pathlib import Path
import re
import subprocess
import sys

app = Path(sys.argv[1]).resolve()
lib = app / 'usr/lib'
plugins = set('coreelements app debugutilsbad subenc subparse videoconvertscale videoconvert videoscale scaletempo id3demux apetag taglib audiomixer interleave playback audioconvert audioresample typefindfunctions autodetect soup mpg123 opus vorbis ogg matroska isomp4 libav faad flac wavparse wavpack alaw mulaw audioparsers volume pulseaudio alsa pipewire jack ossaudio oss4audio equalizer audiofx'.split())
removed = []
for folder in (lib / 'gstreamer-1.0', lib / 'gstreamer1.0/gstreamer-1.0'):
    for file in folder.glob('libgst*.so'):
        # The helper directory must not contain duplicate plugins.
        if folder.name != 'gstreamer-1.0' or folder.parent.name == 'gstreamer1.0' or file.name[6:-3] not in plugins:
            removed.append(file)

env = {**os.environ, 'LD_LIBRARY_PATH': str(lib)}
def dependencies(files):
    result = set()
    for file in files:
        output = subprocess.run(['ldd', str(file)], env=env, capture_output=True, text=True).stdout
        for match in re.findall(r'=> (/.+?) \(', output):
            path = Path(match).resolve()
            if path.is_relative_to(lib):
                result.add(path)
    return result

candidates = dependencies(removed)
for file in removed:
    file.unlink()
# Executables, plugins, GIO/GTK modules and WebKit injected bundles are roots.
roots = []
for file in (app / 'usr').rglob('*'):
    if file.is_file() and file.parent != lib:
        with file.open('rb') as stream:
            if stream.read(4) == b'\x7fELF':
                roots.append(file)
# Graphics libraries can be loaded dynamically by GTK/WebKit.
roots.extend(p for p in lib.glob('*.so*') if re.match(r'lib(EGL|GL|GLES|OpenGL|vulkan|gbm|drm)', p.name))
needed = dependencies(roots) | {p.resolve() for p in roots}
for file in lib.iterdir():
    if file.is_file() and file.resolve() in candidates - needed:
        file.unlink()
print(f'Removed {len(removed)} unnecessary or duplicate GStreamer plugins')
