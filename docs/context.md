# Project context

- Local Tauri 2 application with a Svelte frontend and Rust backend. Linux x64 is the initial target; Windows support comes later.
- The app must manage its own tools without relying on user-installed yt-dlp, FFmpeg, or Python. Future releases should include the required executables in a single package; AppImage packaging and GitHub releases are deferred.
- For now, local setup downloads a pinned official yt-dlp build and verifies its SHA-256. Rust runs this private copy using an explicit path, off the UI thread.
- The first implemented flow is: enter a song/artist → Rust `search_song` command → yt-dlp `ytsearch1` lookup → return `{ id, title }` → display the first result's title, with loading and error states.
- Clicking **Ouvir** calls Rust `preview_song` to resolve a temporary HTTPS audio URL with the private yt-dlp, preferring M4A. Native audio controls handle playback without saving an audio file; retry refreshes the URL and a new search stops playback. The download icon is disabled for now.
- Preview extraction uses reserved cancellation tickets, runs one process at a time, and has a 45-second deadline. Superseding a preview kills and reaps its process group on Linux; stale cancellation tickets cannot affect a newer preview.
- FFmpeg/ffprobe and a bundled Deno are deferred. Preview disables system JavaScript runtimes; some videos may require Deno, and playback depends on Linux WebView codec support.
- `npm run dev` starts Tauri and Vite. Search works in the desktop window; `npm run dev:web` serves only the frontend, without access to Rust.
- A sidebar component uses a native folder picker to list direct regular MP3 files (case-insensitive extension, no recursion or symlinks), with filenames as titles and local play/pause controls. Local playback and YouTube previews stop each other. Folder selection lasts for the current session.
- Icons use `@iconify/svelte` with bundled `andrsrxn` icon modules for offline use. Rust grants asset-protocol access to listed MP3 files; its initial scope is empty.
