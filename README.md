<div align="center">

<img src="static/favicon.png" alt="Open Tunely icon" width="96" />

# Open Tunely

**Find a song, listen, and make it part of your own music library.**

A desktop music downloader and player built with Tauri, Svelte, and Rust.

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-backend-DEA584?logo=rust&logoColor=black)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20x64-FCC624?logo=linux&logoColor=black)](#getting-started)
[![Repository views](https://hits.dwyl.com/guicoelhodev/open-tunely.svg)](https://hits.dwyl.com/guicoelhodev/open-tunely)

[Features](#features) · [Getting started](#getting-started) · [Development](#development) · [Contributing](#contributing)

</div>

> **Early development:** Linux x64 is the current validated platform. Distribution targets are Linux AppImage and Windows x64 NSIS installers; desktop behavior must be validated on each platform. Debian packaging is deferred.

## Features

- **Search and preview** — find the first YouTube result for a song, artist, or video URL and listen before downloading.
- **Download to MP3** — follow progress in a background queue, then retry or remove failed downloads.
- **Your local library** — choose a folder, browse its MP3 files, and rename or delete tracks.
- **One shared player** — play local tracks and YouTube previews with seeking, volume, shuffle, and previous/next controls.
- **Make it yours** — choose a color theme and use the interface in English, Brazilian Portuguese, or Spanish. Your preferences are saved locally.
- **Managed media tools** — use project-managed yt-dlp, FFmpeg, ffprobe, and Deno binaries without installing them globally or setting up Python.

## Getting started

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

## Using Open Tunely

1. Search for a song, artist, or YouTube video URL and preview the result.
2. Choose a music folder, or select **Download MP3** and choose a folder when prompted.
3. Follow progress in the library's download queue. Downloads run one at a time, and you can keep searching while they run.
4. Play tracks from your library. Right-click a track to rename or delete it.

Files are saved as `Track title.mp3`. Existing files with the same name are not overwritten. The library lists MP3 files directly inside the selected folder, excluding subfolders and symbolic links; titles come from filenames. The selected folder is saved locally and restored when you reopen the app.

## How it works

| Layer | Tools and responsibilities |
| --- | --- |
| Interface | Svelte 5, TypeScript, Tailwind CSS, Iconify, and Paraglide for translations |
| Desktop backend | Tauri 2 and Rust commands, with explicit paths to managed media tools |
| Playback | A shared player for local files and temporary YouTube audio URLs; local files stream through a loopback HTTP server with seeking support |
| Downloads | A frontend queue tracks progress while Rust stages conversion and publishes completed MP3 files |

See [the project context](docs/context.md) for more architectural details.

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

Run the **Desktop packages** workflow manually from GitHub Actions using **Run workflow**. It builds Linux x64 AppImage and Windows x64 NSIS (`-setup.exe`) packages. There are no push or tag triggers, and no Debian package is generated.

Download `open-tunely-linux-x64` or `open-tunely-windows-x64` from the completed run's artifacts. When running from the default branch, enable **Create a draft prerelease (default branch only)** to also create a draft GitHub prerelease with both installers and SHA-256 checksums. A unique test tag identifies the exact workflow commit; creating it does not trigger another build. Draft releases remain unpublished until reviewed. Branch builds upload artifacts but skip automatic release creation: GitHub restricts release tokens when the target commit changes workflows relative to the default branch. See [GitHub release permissions](https://docs.github.com/en/rest/releases/releases#create-a-release).

The Linux runner uses Ubuntu 22.04 with Tauri and GStreamer dependencies. It builds the AppImage, applies `scripts/fix-appimage-runtime.sh`, repacks it, and runs `scripts/check-appimage.sh`. Locally, use `npm run tauri build -- --bundles appimage` after preparing tools. For runtime fixes, apply the same postprocessing steps as the workflow. Make AppImages executable before opening; use `--appimage-extract-and-run` when FUSE is unavailable.

The Windows runner uses the MSVC Rust toolchain and MSYS2 UCRT64 to compile the same minimal FFmpeg/ffprobe feature set. It then downloads checksum-verified Windows yt-dlp and Deno binaries, checks offline MP3 conversion, and builds an NSIS installer. For Windows development, compile FFmpeg first with `bash scripts/build-ffmpeg.sh` inside MSYS2 UCRT64, then run `npm run setup:tools` from Windows Node.js. Setup expects `ffmpeg.exe`, `ffprobe.exe`, and their license file in `src-tauri/binaries/`.

Windows installers are unsigned. WebView2 is downloaded during installation if needed, so installation can require internet access. Test search, preview, MP3 download, folder selection, local playback, seeking, renaming, and deletion on Windows before publishing a release. CI conversion checks do not establish desktop playback compatibility.

The FFmpeg build supports AAC, Opus, Vorbis, MP3, FLAC, and basic PCM inputs with MP3 output. yt-dlp handles network downloads. Set `JOBS` to control compilation parallelism or `FFMPEG_OUTPUT` to select an output directory. Build configuration and license texts accompany the tools. Keep Linux releases on the oldest supported build distribution to avoid raising glibc requirements.

Frontend services live in `src/lib/services/`, UI components in `src/lib/components/`, translations in `src/languages/`, and backend code in `src-tauri/src/`.

## Current limitations

- Windows desktop behavior and cross-distribution Linux playback require manual validation. CI produces test artifacts; successful packaging does not replace desktop testing.
- YouTube extraction depends on yt-dlp compatibility and video availability.
- Playback depends on the platform WebView and available codecs.
- Duplicate detection uses the destination filename, not a persistent video ID index.

## Contributing

Bug reports and focused contributions are welcome. When reporting a problem, include your Linux distribution, steps to reproduce, and relevant error output. For changes, describe the expected behavior and run the relevant checks above.

## Acknowledgments

Built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/), [yt-dlp](https://github.com/yt-dlp/yt-dlp), [FFmpeg](https://ffmpeg.org/), [Tailwind CSS](https://tailwindcss.com/), and [Iconify](https://iconify.design/). Managed media tools retain their upstream licenses; the setup includes FFmpeg, LAME, and Deno license texts alongside their binaries.
