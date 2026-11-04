# Project context

- Local Tauri 2 app with Svelte 5 and a Rust backend. Linux x64 is the initial target; Windows validation, AppImage packaging, and GitHub releases are deferred.
- The app manages private yt-dlp and FFmpeg/ffprobe binaries instead of relying on user-installed tools or Python. Local setup downloads pinned builds and verifies SHA-256 checksums; future releases should bundle everything in one package.
- `npm run dev` starts Tauri and Vite. `npm run dev:web` serves only the frontend and cannot invoke Rust. Keep the Tauri JavaScript packages and Rust crates on matching major/minor versions.
- Search uses yt-dlp `ytsearch1` through Rust and returns one `{ id, title }` result. The result card can be dismissed without affecting queued downloads.
- YouTube previews resolve a temporary HTTPS audio URL, preferring M4A, and use native audio controls. Extraction is serialized, uses cancellation tickets, and has a 45-second deadline. Linux cancellation terminates the process group; stale tickets cannot cancel newer previews.
- A bundled Deno is deferred. Preview and download disable system JavaScript runtimes; some videos may require Deno. Playback still depends on the platform WebView and codecs.
- The library uses a native folder picker and lists direct, regular MP3 files only: case-insensitive extension, no recursion or symlinks. Filenames provide titles; the folder label shows its last two segments on one line. Selection lasts for the session.
- A debug-only `dev-library.local.json` can set `music_folder` to avoid repeated selection. It is excluded locally through `.git/info/exclude` and must not be committed.
- Library cards support play/pause and a right-click menu with Rename and Delete. Both actions use a confirmation/edit dialog; Rust validates the selected folder and file, preserves `.mp3` on rename, and prevents overwriting another file.
- Local playback now uses a Rust HTTP server bound to `127.0.0.1` on a dynamic port, with GET/HEAD and byte-range support. Random per-track URLs are revoked when discarded; requests validate that the MP3 remains in the selected folder. This replaced Blob playback after audible stuttering on Linux; the user confirmed the HTTP approach works. Windows remains untested.
- Local playback and YouTube previews stop each other. Temporary playback diagnostics were removed after testing.
- Downloads save MP3s into the selected folder, opening the same picker if needed. Rust uses private FFmpeg, stages conversion, publishes without overwriting, and cleans temporary files. One download runs at a time with a 10-minute deadline.
- A shared download queue decouples searching from downloading and reports live progress. Its summary sits at the bottom of the library and opens a floating list; failed items can be retried or removed. The queue disappears when all downloads finish successfully.
- Reusable operations and shared stores live in `src/lib/services/*.service.ts`. Components own presentation and playback state.
- Frontend styling uses Tailwind CSS through its Vite plugin. `ui/Popover.svelte` and `ui/Modal.svelte` share native overlay structures and accept injected `children`; callers provide content and interaction rules. Icons use bundled Iconify `andrsrxn` modules for offline use.
- Standard validation: `npm run check`, `npm run build`, Rust tests, and Rust formatting checks. Automated checks do not replace playback or platform-specific testing.
