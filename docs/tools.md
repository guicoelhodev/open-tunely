# Media tool requirements

This document defines the media capabilities required by Open Tunely and the compatibility constraints for smaller Linux AppImage and `.deb` packages and Windows distributions.

## Application features

| Feature | Required behavior | Dependencies |
| --- | --- | --- |
| Search by song or artist | Resolve `ytsearch1:<query>` to one JSON result containing `id`, `title`, and optional `channel` | yt-dlp YouTube search and JSON output; configured Deno runtime |
| Search by URL | Validate YouTube/youtu.be links, normalize watch, shorts, embed, and live links to one video, and extract metadata | yt-dlp YouTube video extraction; configured Deno runtime |
| Prepare a preview | Return one HTTPS audio URL, preferring M4A with an HTTPS audio fallback | yt-dlp format selection, URL extraction, and Deno integration |
| Play a preview | Play, pause, seek, adjust volume, and advance using the remote URL in an HTML audio element | WebView media stack |
| Download MP3 | Download `bestaudio/best`, extract audio, convert to MP3 at quality `0`, and report progress | yt-dlp, Deno, FFmpeg, and ffprobe |
| Select a folder and list tracks | Find local MP3 files without converting other formats | Native dialog and Rust filesystem operations |
| Play local MP3 | Serve `audio/mpeg` over local HTTP with byte ranges for seeking | Rust HTTP server and WebView media stack |
| Rename or delete tracks | Modify validated local files and refresh the library | Rust filesystem operations |

The backend invokes yt-dlp for search, preview preparation, and download. FFmpeg conversion commands are supplied by yt-dlp postprocessors. Local playback and library operations do not invoke the bundled FFmpeg or yt-dlp executables.

The supported feature set does not include video export, subtitles, embedded artwork, volume normalization, trimming, local batch conversion, or output codecs other than MP3. Playlist links containing a supported video are normalized to that video. Other websites are rejected.

## yt-dlp command contract

Every invocation uses these isolation and runtime options:

```text
--ignore-config --no-plugin-dirs --no-playlist
--no-js-runtimes --js-runtimes deno:<private-runtime-path>
```

User configuration and external plugins must not change application behavior. Arguments and paths must remain separate process arguments, including paths containing spaces.

### Search

```text
--flat-playlist --skip-download --dump-json
--socket-timeout 15 --retries 1 --extractor-retries 1
-- <target>
```

The target is a textual `ytsearch1:` query or a normalized YouTube video URL. The application reads the first nonempty JSON line and requires a nonempty ID and title. Both search modes require validation. Search uses process output collection without the process deadline applied to previews and downloads.

Implementation: `search_target`, `search_with_ytdlp`, and `parse_song` in [lib.rs](../src-tauri/src/lib.rs).

### Preview

```text
--skip-download --get-url --check-formats
--format bestaudio[ext=m4a][protocol=https]/bestaudio[protocol=https]
--socket-timeout 15 --retries 1 --extractor-retries 1
-- https://www.youtube.com/watch?v=<id>
```

The result must contain exactly one HTTPS URL. Selected formats are checked for downloadability before returning the URL. Extraction is serialized, cancellable, and subject to a 45-second process deadline. A rejected preview stream is refreshed once during initial playback; extraction failures and autoplay permission errors are not retried. The WebView downloads and decodes the preview; reducing bundled FFmpeg codecs does not reduce WebView playback requirements.

Implementation: `preview_with_ytdlp`, `parse_preview_url`, and `preview_song` in [lib.rs](../src-tauri/src/lib.rs), [music.service.ts](../src/lib/services/music.service.ts), and [Player.svelte](../src/lib/components/Player.svelte).

### Download

```text
--newline --progress --progress-delta 0.25
--progress-template download:QUEUE:%(progress._percent_str)s
--progress-template postprocess:CONVERTING
--no-continue --no-overwrites
--socket-timeout 15 --retries 2 --extractor-retries 1
--match-filters !is_live
--format bestaudio/best
--extract-audio --audio-format mp3 --audio-quality 0
--output %(title).150B.%(ext)s
--ffmpeg-location <private-tools-directory>
-- https://www.youtube.com/watch?v=<id>
```

Preserve progress parsing from both stdout and stderr, including `QUEUE:` percentages and the `CONVERTING` marker. Downloads run one at a time with a 600-second deadline. Staging occurs inside the destination directory. Publication requires exactly one nonempty MP3 and must not overwrite an existing file. Source and partial files are removed after success or failure. Unix process supervision terminates the process group so child processes are also stopped.

**`bestaudio/best` does not constrain codecs, containers, or protocols.** The `best` fallback can contain video, so its audio track must remain extractable. A reduced build must support the selected inputs and postprocessors, or introduce an explicit format policy with availability and quality checks.

Implementation: `download_command`, `download_with_ytdlp`, `publish_mp3`, and `download_song` in [download.rs](../src-tauri/src/download.rs), process supervision in [lib.rs](../src-tauri/src/lib.rs), and [download.service.ts](../src/lib/services/download.service.ts).

## FFmpeg capabilities

[build-ffmpeg.sh](../scripts/build-ffmpeg.sh) defines the build configuration. It uses `--disable-everything --disable-autodetect --disable-network --enable-small`, static libraries, and libmp3lame as the requested encoder. It builds both **ffmpeg and ffprobe**, with ffplay, device capture, and swscale disabled.

| Capability | Build components | Requirement |
| --- | --- | --- |
| MP3 encoding | `libmp3lame` encoder and `mp3` muxer | Required output format and quality selection |
| AAC in M4A/MP4 | `mov` demuxer; `aac` decoder and parser | YouTube audio input coverage |
| Opus/Vorbis in WebM | `matroska` demuxer; `opus,vorbis` decoders and parsers | Alternative audio input coverage |
| Additional audio inputs | `ogg,mp3,aac,flac,wav` demuxers; `mp3,flac,pcm_s16le,pcm_s24le,pcm_f32le` decoders; `mpegaudio,flac` parsers | Defensive input coverage; PCM s16le WAV is also the offline test fixture |
| Sample format and rate conversion | `aresample,aformat,anull` filters and configure-selected dependencies | Audio conversion pipeline, including implicitly inserted filters |
| File and pipe access | `file,pipe` protocols | File-based staging/conversion; pipe support retained by the build without an explicit Rust invocation |
| Postprocessor compatibility | `mp4,ipod` muxers and `aac_adtstoasc` bitstream filter | The build script records yt-dlp passing `-movflags` even for MP3 output and performing M4A fixups; preserve until those paths pass without these components |
| Media inspection | `ffprobe` executable | Required by the backend before downloading, available for yt-dlp discovery, and used by conversion validation |

The additional input formats are not a local file import feature. Their removal changes input coverage and requires explicit validation. A format enabled in the build is not proof that every corresponding conversion path has been tested.

[ffmpeg-build-config.txt](../src-tauri/binaries/ffmpeg-build-config.txt) records generated configuration, including automatically selected dependencies such as the MOV muxer. The table lists requested components rather than every resulting component. Change the build script and rebuild instead of editing generated configuration.

No application feature directly requires video encoding, GPU acceleration, capture devices, video filters, or ffplay. Network-disabled FFmpeg supports the file-based conversion path, but fragmented downloads, HLS/DASH, or yt-dlp fixups can select other paths. Record the actual downloader and postprocessor commands when validating protocol changes. Preview HTTPS playback belongs to the WebView and does not itself require HTTPS support in bundled FFmpeg.

## yt-dlp and JavaScript runtime capabilities

A specialized yt-dlp distribution must retain:

- YouTube video and search extraction, metadata retrieval, and JSON serialization.
- Format selection and fallback, direct audio URL resolution, and downloads of selected formats, including fragments when selected.
- FFmpeg/ffprobe discovery, audio extraction, MP3 conversion, and required fixups.
- Progress output and all command options defined above.
- JavaScript runtime integration used by YouTube extraction.

[setup-tools.mjs](../scripts/setup-tools.mjs) defines tool versions, release artifacts, and integrity hashes. The application passes its private Deno path to yt-dlp and requires the Deno executable in all three workflows. It does not run its own Deno scripts. Runtime removal or substitution requires coordinated code changes and validation of search, URL lookup, preview, and download. Runtime equivalence must be demonstrated against the packaged yt-dlp release.

Other website extractors, external plugins, global configuration, login/cookies, subtitles, and thumbnail downloads are not requested by the interface. Removing internal modules still requires dependency checks: unused interface features do not prove that their implementation modules can be deleted without affecting imports or YouTube extraction.

## Packaging requirements

Measure executables, WebKit/GTK, GStreamer, and other AppDir libraries separately when choosing size reductions. Uncompressed executable sizes and compressed package contributions are different measurements. Compare both for each candidate build.

### Linux AppImage and .deb

[tauri.conf.json](../src-tauri/tauri.conf.json) includes yt-dlp, Deno, FFmpeg, ffprobe, and license files as resources under `binaries/`. `private_tool` in [lib.rs](../src-tauri/src/lib.rs) resolves `src-tauri/binaries` in debug builds and `resource_dir()/binaries` in release builds. There is no system `PATH` fallback; this contract also applies to `.deb` installations.

Preserve executable permissions, target architecture, paths containing spaces, and dynamic dependencies compatible with the minimum supported distribution. Static FFmpeg libraries do not establish independence from the system loader or libc. Preserve distributed license notices when replacing builds.

Linux playback also depends on WebKit/GStreamer. [check-appimage.sh](../scripts/check-appimage.sh) requires `gst-plugin-scanner` and these plugins:

```text
debugutilsbad subenc id3demux audiofx playback audioconvert
audioresample typefindfunctions autodetect soup mpg123
opus vorbis matroska isomp4
```

AAC decoding requires either the `libav` or `faad` plugin. These are the verifier's requirements, not proof that each plugin is individually minimal. Bundled FFmpeg and GStreamer serve different workflows; removing one because the other is present can break playback.

[fix-appimage-runtime.sh](../scripts/fix-appimage-runtime.sh) sources GTK/GStreamer hooks before setting bundled library paths, then executes the native application directly. Avoid starting a host shell after adding bundled libraries: its readline dependency may be incompatible. The script also removes Python bindings unnecessary for HTML audio. Validate playback after changing runtime libraries or plugin selection.

Using distribution-provided tools in a `.deb` requires changing path resolution and declaring package dependencies and compatible versions. Removing bundled resources alone breaks tool discovery. Test installed packages outside the checkout.

### Windows

Windows resources use `.exe` names for yt-dlp, Deno, FFmpeg, and ffprobe, configured in [tauri.windows.conf.json](../src-tauri/tauri.windows.conf.json). Tool resolution appends the platform executable suffix. The setup downloads checksum-verified Windows x64 yt-dlp and Deno artifacts and expects FFmpeg/ffprobe to have been compiled under MSYS2 first. FFmpeg uses static MinGW linking to avoid depending on the build machine's runtime DLLs.

The NSIS installer downloads WebView2 when needed. Tools run without console windows; cancellation and deadline cleanup terminate an owned Windows Job Object, including descendants retaining output pipes. Validate cleanup and playback on Windows independently of Linux GStreamer packaging.

## Verification requirements

`npm run check:tools` runs [check-media-tools.mjs](../scripts/check-media-tools.mjs), exercising **yt-dlp → FFmpeg → MP3** offline with a one-second PCM s16le WAV fixture. It checks MP3 codec and duration using:

```text
ffprobe -v error -show_streams -show_format -of json <output>
```

This catches indirect postprocessor option failures such as `-movflags` compatibility. It does not cover YouTube extraction, Deno, AAC/M4A, WebM/Opus, M4A fixups, or playback.

Validate a candidate tool set against the following scenarios:

| Scenario | Required checks |
| --- | --- |
| Text search and each accepted URL family | Valid result and preserved invalid-input rejection |
| M4A preview and HTTPS audio fallback | Playback, pause, seek, volume, rapid track changes, and cancellation |
| AAC/M4A, Opus/WebM, and Vorbis where available | Valid MP3 conversion; record selected formats and actual postprocessor commands |
| MP3, Ogg, FLAC, and PCM input coverage | Offline fixtures for retained coverage; explicitly document any removed coverage |
| M4A fixups, fragmented audio, and `best` fallback | Exercise selected paths; identify untested paths rather than assuming coverage |
| Local library | List, play MP3, seek, advance, rename, and delete |
| Failure and publication handling | Network failure, live input, existing destination, unwritable folder, timeout, and staging/child-process cleanup |
| Final packages | Run AppImage and installed `.deb` in clean environments without media tools on `PATH`; use paths with spaces and non-ASCII characters |

Rust tests in [lib.rs](../src-tauri/src/lib.rs) and [download.rs](../src-tauri/src/download.rs) cover argument contracts, validation, progress, pipes, deadlines, and publication. Tests using simulated processes do not replace real-tool integration checks.

`scripts/check-appimage.sh <package.AppImage>` verifies payload integrity, executable versions, and plugin presence. `scripts/check-appimage-startup.sh <package.AppImage>` checks that the final package stays running for 15 seconds under D-Bus and Xvfb without loader errors. Interactive playback still requires desktop testing.

For each size-reduction experiment, retain versions and hashes, effective FFmpeg configuration, executable and compressed package sizes, postprocessor commands, and verification results. Revalidate when changing yt-dlp, format policy, JavaScript runtime, or player features. Treat the build configuration as a compatibility baseline rather than a proven absolute minimum.
