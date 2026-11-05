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

> **Early development:** Linux x64 is the initial development target. Windows support and standalone releases are planned.

## Features

- **Search and preview** — find the first YouTube result for a song, artist, or video URL and listen before downloading.
- **Download to MP3** — follow progress in a background queue, then retry or remove failed downloads.
- **Your local library** — choose a folder, browse its MP3 files, and rename or delete tracks.
- **One shared player** — play local tracks and YouTube previews with seeking, volume, shuffle, and previous/next controls.
- **Make it yours** — choose a color theme and use the interface in English, Brazilian Portuguese, or Spanish. Your preferences are saved locally.
- **Managed media tools** — use project-managed yt-dlp, FFmpeg, ffprobe, and Deno binaries without installing them globally or setting up Python.

## Getting started

### Requirements

- Linux x64
- Node.js and npm
- The Rust toolchain and [Tauri's Linux prerequisites](https://v2.tauri.app/start/prerequisites/)
- Internet access, `curl`, `tar` with XZ support, `unzip`, GCC, make, pkg-config and NASM for media tool setup

### Run locally

```sh
git clone https://github.com/guicoelhodev/open-tunely.git
cd open-tunely
npm install
npm run setup:tools
npm run dev
```

The setup script downloads pinned yt-dlp and Deno builds and compiles audio-only FFmpeg/ffprobe from checksum-verified FFmpeg 8.0 and LAME 3.100 sources. Tools are stored in `src-tauri/binaries/`. FFmpeg is rebuilt on each setup; compiler dependencies are needed only on the build machine.

To build only FFmpeg/ffprobe, run `bash scripts/build-ffmpeg.sh`. It keeps AAC, Opus, Vorbis, MP3, FLAC and basic PCM decoding, MP3 encoding and local-file processing. Media libraries are linked statically. Network access and video decoding are disabled; yt-dlp handles downloads. The same recipe targets Windows through MSYS2 MINGW64, but Windows execution and integration are not yet validated. Use `FFMPEG_OUTPUT` to choose a separate output directory and `JOBS` to control build parallelism. Build configuration and upstream license texts accompany the output. Linux builds should run on the oldest supported distribution to avoid requiring a newer glibc.

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
| `npm run setup:tools` | Download and verify the managed media tools |
| `npm run check` | Check Svelte and TypeScript |
| `npm run build` | Build the frontend |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Run backend tests |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Check Rust formatting |

Frontend services live in `src/lib/services/`, UI components in `src/lib/components/`, translations in `src/languages/`, and backend code in `src-tauri/src/`.

## Current limitations

- Windows playback and packaging have not been validated. AppImage packaging and public releases are deferred.
- YouTube extraction depends on yt-dlp compatibility and video availability.
- Playback depends on the platform WebView and available codecs.
- Duplicate detection uses the destination filename, not a persistent video ID index.

## Contributing

Bug reports and focused contributions are welcome. When reporting a problem, include your Linux distribution, steps to reproduce, and relevant error output. For changes, describe the expected behavior and run the relevant checks above.

## Acknowledgments

Built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/), [yt-dlp](https://github.com/yt-dlp/yt-dlp), [FFmpeg](https://ffmpeg.org/), [Tailwind CSS](https://tailwindcss.com/), and [Iconify](https://iconify.design/). Managed media tools retain their upstream licenses; the setup includes the FFmpeg and Deno licenses alongside their binaries.
