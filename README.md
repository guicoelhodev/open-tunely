<div align="center">

<img src="static/brand/orbit.svg" alt="Open Tunely" width="104" height="104" />

# Open Tunely

**Find a song. Take a listen. Make it part of your library.**

A free desktop app for discovering music and enjoying your own MP3 collection.

### [Download for Windows (64-bit)](https://github.com/guicoelhodev/open-tunely/releases/download/latest/Open-Tunely-windows-x64-setup.exe)

### [Download for Linux (AppImage)](https://github.com/guicoelhodev/open-tunely/releases/download/latest/Open-Tunely-linux-x64.AppImage)

Choose your operating system above to download directly. No GitHub account needed.

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-backend-DEA584?logo=rust&logoColor=black)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%26%20Linux%20x64-0891b2)](#get-started)
[![Repository views](https://hits.dwyl.com/guicoelhodev/open-tunely.svg)](https://hits.dwyl.com/guicoelhodev/open-tunely)

[Features](#make-yourself-at-home) · [Get started](#get-started) · [All downloads](https://github.com/guicoelhodev/open-tunely/releases/tag/latest) · [Feedback](https://github.com/guicoelhodev/open-tunely/issues)

</div>

## Make yourself at home

| Discover | Build your library | Press play |
| --- | --- | --- |
| Search by song, artist, or YouTube link. Preview a result before saving it. | Save permitted audio as MP3, choose your folder, and follow downloads in the queue. | Enjoy local tracks with seeking, volume, shuffle, and previous/next controls. |

- **Your files, your folder.** Browse your MP3s and rename or remove tracks from the app.
- **Your colors.** Pick a light or dark appearance with cyan, indigo, pink, or neutral accents.
- **Your language.** Available in English, Brazilian Portuguese, and Spanish.
- **Ready to run.** No Python, FFmpeg setup, or developer tools required.

## Get started

1. Download the app for **Windows x64** or **Linux x64** using the links above. No GitHub account needed.
2. Open the Windows installer, or make the Linux AppImage executable and open it.
3. Search for something to listen to, or choose a folder to enjoy your existing MP3 library.

<details>
<summary>Opening the AppImage on Linux</summary>

```sh
chmod +x Open-Tunely-linux-x64.AppImage
./Open-Tunely-linux-x64.AppImage
```

If your system does not provide FUSE:

```sh
./Open-Tunely-linux-x64.AppImage --appimage-extract-and-run
```

</details>

The Windows installer is unsigned and may show a security prompt. It downloads WebView2 if your system needs it. Internet access is required for search, online previews, and downloads.

**Early release:** help us improve by [reporting a problem](https://github.com/guicoelhodev/open-tunely/issues). Include your operating system, what happened, and the steps to reproduce it.

Download only content you own or have permission to save. Open Tunely is an independent project and is not affiliated with YouTube or Google.

---

Want to contribute or run the project locally? See the [development guide](docs/development.md).

Built with [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/), [yt-dlp](https://github.com/yt-dlp/yt-dlp), and [FFmpeg](https://ffmpeg.org/). Orbit artwork uses [Lucide](https://lucide.dev/) under its [license](docs/licenses/lucide.txt). Bundled tools retain their upstream licenses.
