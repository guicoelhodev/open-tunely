#!/usr/bin/env bash
# Linux native or Windows via MSYS2 MINGW64 (GCC, make, nasm, curl, tar).
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
prefix="$work/install"
fetch() {
  curl --fail --location --retry 3 "$1" -o "$work/$2"
  echo "$3  $work/$2" | sha256sum --check -
}
fetch https://ffmpeg.org/releases/ffmpeg-8.0.tar.xz ffmpeg.tar.xz b2751fccb6cc4c77708113cd78b561059b6fa904b24162fa0be2d60273d27b8e
fetch https://downloads.sourceforge.net/project/lame/lame/3.100/lame-3.100.tar.gz lame.tar.gz ddfe36cab873794038ae2c1210557ad34857a4b6bdc515785d1da9e175b1da1e
tar -xf "$work/ffmpeg.tar.xz" -C "$work"
tar -xf "$work/lame.tar.gz" -C "$work"
cd "$work/lame-3.100"
./configure --prefix="$prefix" --disable-shared --enable-static --disable-frontend
make -j"${JOBS:-4}"
make install
cd "$work/ffmpeg-8.0"
PKG_CONFIG_PATH="$prefix/lib/pkgconfig" ./configure \
  --disable-everything --disable-autodetect --disable-network \
  --disable-doc --disable-debug --disable-ffplay --disable-avdevice --disable-swscale \
  --enable-small --enable-static --disable-shared --enable-libmp3lame \
  --enable-encoder=libmp3lame \
  --enable-decoder=aac,opus,vorbis,mp3,flac,pcm_s16le,pcm_s24le,pcm_f32le \
  --enable-demuxer=mov,matroska,ogg,mp3,aac,flac,wav \
  --enable-muxer=mp3 --enable-protocol=file,pipe \
  --enable-filter=aresample,aformat,anull --enable-parser=aac,opus,vorbis,mpegaudio,flac \
  --extra-cflags="-I$prefix/include" --extra-ldflags="-L$prefix/lib"
make -j"${JOBS:-4}" ffmpeg ffprobe
suffix=""
case "$(uname -s)" in MINGW*|MSYS*) suffix=.exe ;; esac
out="${FFMPEG_OUTPUT:-$root/src-tauri/binaries}"
mkdir -p "$out"
cp "ffmpeg$suffix" "ffprobe$suffix" "$out/"
cat COPYING.LGPLv2.1 "$work/lame-3.100/COPYING" > "$out/ffmpeg-LICENSE.txt"
cp ffbuild/config.mak "$out/ffmpeg-build-config.txt"
ls -lh "$out/ffmpeg$suffix" "$out/ffprobe$suffix"
