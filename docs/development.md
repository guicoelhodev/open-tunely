# Development

## Local setup

### Requirements

- Linux x64, or Windows x64 with MSVC and WebView2 prerequisites
- Node.js and npm
- The Rust toolchain and [Tauri's Linux prerequisites](https://v2.tauri.app/start/prerequisites/)
- Internet access, Bash, `curl`, `tar` with Gzip/XZ support, `unzip`, `sha256sum`, GCC, make, pkg-config, and NASM for media tool setup; `patchelf` for AppImage packaging

### Run locally

```sh
git clone https://github.com/guicoelhodev/open-tunely.git
cd open-tunely
npm install
npm run setup:tools
npm run dev
```

The setup script downloads checksum-verified yt-dlp and Deno binaries and compiles FFmpeg/ffprobe with LAME for MP3 conversion. Tools are stored in `src-tauri/binaries/` and included in desktop packages. Build tools are required only on the development machine. FFmpeg/ffprobe are rebuilt each time setup runs.

`npm run dev` opens the Tauri desktop window. Search, playback, and filesystem operations require that window. For frontend-only visual work, use `npm run dev:web`.

## Development

| Command | Purpose |
| --- | --- |
| `npm run dev` | Start the desktop app in development mode |
| `npm run dev:web` | Start the frontend only |
| `npm run setup:tools` | Download yt-dlp and Deno and compile FFmpeg/ffprobe |
| `bash scripts/build-ffmpeg.sh` | Compile only FFmpeg/ffprobe |
| `npm run check:tools` | Validate offline yt-dlp to MP3 conversion |
| `npm run check` | Check Svelte and TypeScript |
| `npm run build` | Build the frontend |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Run backend tests |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Check Rust formatting |

### Desktop packages

The **Desktop packages** workflow builds AppImage and Windows x64 NSIS installers on every push to `master`, including merges. Manual runs are also available. Branch runs upload Actions artifacts for testing; only `master` publishes public downloads.

Manual runs build the selected existing version tag and upload both platform packages to its GitHub Release. After both uploads succeed, the release is marked latest. Fixed asset names allow the README links to follow that release automatically.

Stable links:

- `https://github.com/guicoelhodev/open-tunely/releases/latest/download/Open-Tunely-windows-x64-setup.exe`
- `https://github.com/guicoelhodev/open-tunely/releases/latest/download/Open-Tunely-linux-x64.AppImage`

The Linux runner uses Ubuntu 22.04 with Tauri and GStreamer dependencies. It builds the AppImage, applies `scripts/fix-appimage-runtime.sh`, repacks it, and runs `scripts/check-appimage.sh` plus a virtual-display startup smoke test. Locally, use `npm run tauri build -- --bundles appimage` after preparing tools. For runtime fixes, apply the same postprocessing steps as the workflow. Make AppImages executable before opening; use `--appimage-extract-and-run` when FUSE is unavailable.

The Windows runner uses the MSVC Rust toolchain and MSYS2 UCRT64 to compile the same minimal FFmpeg/ffprobe feature set. It then downloads checksum-verified Windows yt-dlp and Deno binaries, checks offline MP3 conversion, and builds an NSIS installer. For Windows development, compile FFmpeg first with `bash scripts/build-ffmpeg.sh` inside MSYS2 UCRT64, then run `npm run setup:tools` from Windows Node.js. Setup expects `ffmpeg.exe`, `ffprobe.exe`, and their license file in `src-tauri/binaries/`.

Windows installers are unsigned. WebView2 is downloaded during installation if needed, so installation can require internet access. Test search, preview, MP3 download, folder selection, local playback, seeking, renaming, and deletion on Windows before publishing a release. CI conversion checks do not establish desktop playback compatibility.

The FFmpeg build supports AAC, Opus, Vorbis, MP3, FLAC, and basic PCM inputs with MP3 output. yt-dlp handles network downloads. Set `JOBS` to control compilation parallelism or `FFMPEG_OUTPUT` to select an output directory. Build configuration and license texts accompany the tools. Keep Linux releases on the oldest supported build distribution to avoid raising glibc requirements.

