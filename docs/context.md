# Project context

- Local Tauri 2 application with a Svelte frontend and Rust backend. Linux x64 is the initial target; Windows support comes later.
- The app must manage its own tools without relying on user-installed yt-dlp, FFmpeg, or Python. Future releases should include the required executables in a single package; AppImage packaging and GitHub releases are deferred.
- For now, local setup downloads a pinned official yt-dlp build and verifies its SHA-256. Rust runs this private copy using an explicit path, off the UI thread.
- The first implemented flow is: enter a song/artist → Rust `search_song` command → yt-dlp `ytsearch1` lookup → return `{ id, title }` → display the first result's title, with loading and error states.
- This step retrieves metadata only, without downloading audio. FFmpeg/ffprobe and Deno will be added when download and conversion are implemented.
- `npm run dev` starts Tauri and Vite. Search works in the desktop window; `npm run dev:web` serves only the frontend, without access to Rust.
