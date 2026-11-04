use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};

#[derive(Default)]
pub struct DownloadState(Mutex<()>);

fn download_command(binary: &Path, ffmpeg: &Path, work: &Path, id: &str) -> Command {
    let mut command = Command::new(binary);
    command
        .current_dir(work)
        .args([
            "--ignore-config",
            "--no-plugin-dirs",
            "--no-js-runtimes",
            "--no-playlist",
            "--newline",
            "--progress",
            "--progress-delta",
            "0.25",
            "--progress-template",
            "download:QUEUE:%(progress._percent_str)s",
            "--progress-template",
            "postprocess:CONVERTING",
            "--no-continue",
            "--no-overwrites",
            "--socket-timeout",
            "15",
            "--retries",
            "2",
            "--extractor-retries",
            "1",
            "--match-filters",
            "!is_live",
            "--format",
            "bestaudio/best",
            "--extract-audio",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
            "--output",
            "%(title).150B [%(id)s].%(ext)s",
            "--ffmpeg-location",
        ])
        .arg(ffmpeg)
        .arg("--")
        .arg(format!("https://www.youtube.com/watch?v={id}"));
    command
}

fn publish_mp3(work: &Path, folder: &Path) -> Result<PathBuf, String> {
    let mut files = std::fs::read_dir(work)
        .map_err(|_| "Não foi possível ler o MP3 convertido.")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Não foi possível ler o MP3 convertido.")?
        .into_iter()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "mp3"));
    let file = files.next().ok_or("O download não gerou um arquivo MP3.")?;
    if files.next().is_some()
        || !file
            .file_type()
            .map_err(|_| "Não foi possível verificar o MP3.")?
            .is_file()
        || file
            .metadata()
            .map_err(|_| "Não foi possível verificar o MP3.")?
            .len()
            == 0
    {
        return Err("O download não gerou um arquivo MP3 válido.".into());
    }
    let destination = folder.join(file.file_name());
    // On Linux, tempfile first tries renameat2(RENAME_NOREPLACE), which works on
    // FAT/exFAT. Staging is on the destination filesystem; never overwrite a
    // file or follow a destination symlink.
    let temporary = tempfile::TempPath::try_from_path(file.path())
        .map_err(|_| "Não foi possível preparar o MP3 para salvar.")?;
    temporary.persist_noclobber(&destination).map_err(|error| {
        if error.error.kind() == std::io::ErrorKind::AlreadyExists {
            "Esta música já existe na pasta selecionada.".to_string()
        } else {
            "Não foi possível salvar o MP3 na pasta. Verifique o espaço e as permissões.".into()
        }
    })?;
    Ok(destination)
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    percent: Option<f64>,
    converting: bool,
}

#[derive(Default)]
struct ProgressReader {
    buffers: [Vec<u8>; 2],
}

impl ProgressReader {
    fn feed(&mut self, bytes: &[u8], stderr: bool, report: &mut impl FnMut(DownloadProgress)) {
        let buffer = &mut self.buffers[usize::from(stderr)];
        for &byte in bytes {
            if byte == b'\n' || byte == b'\r' {
                let line = String::from_utf8_lossy(buffer);
                if line.trim() == "CONVERTING" {
                    report(DownloadProgress {
                        percent: Some(100.0),
                        converting: true,
                    });
                } else if let Some(percent) = line
                    .trim()
                    .strip_prefix("QUEUE:")
                    .and_then(|value| value.trim().trim_end_matches('%').parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                {
                    report(DownloadProgress {
                        percent: Some(percent.clamp(0.0, 100.0)),
                        converting: false,
                    });
                }
                buffer.clear();
            } else if buffer.len() < 16_384 {
                buffer.push(byte);
            }
        }
    }
}

fn download_with_ytdlp(
    binary: &Path,
    ffmpeg: &Path,
    folder: &Path,
    id: &str,
    mut progress: impl FnMut(DownloadProgress),
) -> Result<PathBuf, String> {
    let work = tempfile::Builder::new()
        .prefix(".downloader-")
        .tempdir_in(folder)
        .map_err(|_| "Não foi possível gravar na pasta. Verifique o espaço e as permissões.")?;
    let mut reader = ProgressReader::default();
    let output = super::run_ytdlp_process_with_progress(
        &mut download_command(binary, ffmpeg, work.path(), id),
        &AtomicBool::new(false),
        Duration::from_secs(600),
        |bytes, stderr| reader.feed(bytes, stderr, &mut progress),
    )?;
    if !output.status.success() {
        eprintln!(
            "yt-dlp download: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err("Não foi possível baixar e converter esta música. Verifique sua conexão e tente novamente.".into());
    }
    publish_mp3(work.path(), folder)
    // TempDir removes source audio and partial files on success or failure.
}

#[tauri::command]
pub async fn download_song(
    app: tauri::AppHandle,
    id: String,
    on_progress: tauri::ipc::Channel<DownloadProgress>,
    state: tauri::State<'_, Arc<DownloadState>>,
) -> Result<String, String> {
    if !super::valid_video_id(&id) {
        return Err("O identificador da música é inválido.".into());
    }
    // Destinations come from native selection or the local development configuration.
    let folder = super::library::selected_folder(&app)?;
    let binary = super::tool_binary(&app)?;
    let ffmpeg = super::private_tool(&app, "ffmpeg")?;
    super::private_tool(&app, "ffprobe")?;
    let state = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = state
            .0
            .try_lock()
            .map_err(|_| "Aguarde o download atual terminar.")?;
        download_with_ytdlp(
            &binary,
            ffmpeg.parent().unwrap(),
            &folder,
            &id,
            |progress| {
                let _ = on_progress.send(progress);
            },
        )
        .map(|path| path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|_| "O download foi interrompido.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_handles_split_lines_both_streams_and_invalid_values() {
        let mut reader = ProgressReader::default();
        let mut updates = Vec::new();
        let mut report = |value| updates.push(value);
        reader.feed(b"[info] ignored\nQUEUE: 2", false, &mut report);
        reader.feed(b"QUEUE: N/A%\nQUEUE: NaN%\n", true, &mut report);
        reader.feed(b"0.5%\nQUEUE:101%\nCONVERT", false, &mut report);
        reader.feed(b"ING\n", false, &mut report);
        assert_eq!(
            updates,
            vec![
                DownloadProgress {
                    percent: Some(20.5),
                    converting: false
                },
                DownloadProgress {
                    percent: Some(100.0),
                    converting: false
                },
                DownloadProgress {
                    percent: Some(100.0),
                    converting: true
                },
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn reports_progress_before_the_process_exits() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf '%s\\n' 'QUEUE:42%'; sleep 2"]);
        let cancelled = AtomicBool::new(false);
        let mut reader = ProgressReader::default();
        let result = super::super::run_ytdlp_process_with_progress(
            &mut command,
            &cancelled,
            Duration::from_secs(5),
            |bytes, stderr| {
                reader.feed(bytes, stderr, &mut |_| {
                    // Cancelling from this callback proves it runs while the child is alive.
                    cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
                })
            },
        );
        assert!(result.unwrap_err().contains("cancelada"));
    }

    #[cfg(unix)]
    #[test]
    fn staging_is_cleaned_after_success_and_failure() {
        use std::os::unix::fs::PermissionsExt;
        let tools = tempfile::tempdir().unwrap();
        let binary = tools.path().join("yt-dlp");
        let folder = tempfile::tempdir().unwrap();
        for (script, succeeds) in [
            (
                "#!/bin/sh\nprintf complete > 'Song.mp3'\nprintf source > source.webm\n",
                true,
            ),
            ("#!/bin/sh\nprintf partial > source.part\nexit 1\n", false),
        ] {
            std::fs::write(&binary, script).unwrap();
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
            let result =
                download_with_ytdlp(&binary, tools.path(), folder.path(), "P-lxOj0XpEE", |_| {});
            assert_eq!(result.is_ok(), succeeds);
            let files = std::fs::read_dir(folder.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect::<Vec<_>>();
            assert_eq!(files, ["Song.mp3"]);
        }
    }

    #[test]
    fn download_uses_explicit_private_tools_and_separate_arguments() {
        let command = download_command(
            Path::new("/private/yt-dlp"),
            Path::new("/private/tools"),
            Path::new("/music with spaces"),
            "P-lxOj0XpEE",
        );
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect();
        assert_eq!(command.get_program(), "/private/yt-dlp");
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new("/music with spaces"))
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--ffmpeg-location", "/private/tools"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--audio-format", "mp3"])
        );
        assert!(args.contains(&"--no-js-runtimes".into()));
        assert_eq!(
            args.last().unwrap(),
            "https://www.youtube.com/watch?v=P-lxOj0XpEE"
        );
    }

    #[test]
    fn publishes_complete_mp3_without_overwriting_existing_files() {
        let folder = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir_in(folder.path()).unwrap();
        std::fs::write(work.path().join("Song.mp3"), b"complete").unwrap();
        let saved = publish_mp3(work.path(), folder.path()).unwrap();
        assert_eq!(std::fs::read(&saved).unwrap(), b"complete");
        assert!(!work.path().join("Song.mp3").exists());
        std::fs::write(work.path().join("Song.mp3"), b"replacement").unwrap();
        std::fs::write(work.path().join("other.mp3"), b"other").unwrap();
        assert!(publish_mp3(work.path(), folder.path()).is_err());
        std::fs::remove_file(work.path().join("other.mp3")).unwrap();
        assert!(
            publish_mp3(work.path(), folder.path())
                .unwrap_err()
                .contains("já existe")
        );
        assert_eq!(std::fs::read(saved).unwrap(), b"complete");
    }

    #[cfg(unix)]
    #[test]
    fn refuses_existing_destination_symlinks_including_dangling_ones() {
        let folder = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir_in(folder.path()).unwrap();
        let target = folder.path().join("target.txt");
        let destination = folder.path().join("Song.mp3");
        for dangling in [false, true] {
            if !dangling {
                std::fs::write(&target, b"original").unwrap();
            }
            std::os::unix::fs::symlink(&target, &destination).unwrap();
            std::fs::write(work.path().join("Song.mp3"), b"replacement").unwrap();
            assert!(
                publish_mp3(work.path(), folder.path())
                    .unwrap_err()
                    .contains("já existe")
            );
            assert!(
                std::fs::symlink_metadata(&destination)
                    .unwrap()
                    .is_symlink()
            );
            if !dangling {
                assert_eq!(std::fs::read(&target).unwrap(), b"original");
                std::fs::remove_file(&target).unwrap();
            } else {
                assert!(!target.exists());
            }
            std::fs::remove_file(&destination).unwrap();
        }
    }

    #[test]
    fn concurrent_publications_keep_the_first_complete_file() {
        let folder = tempfile::tempdir().unwrap();
        let first = tempfile::tempdir_in(folder.path()).unwrap();
        let second = tempfile::tempdir_in(folder.path()).unwrap();
        std::fs::write(first.path().join("Song.mp3"), b"first").unwrap();
        std::fs::write(second.path().join("Song.mp3"), b"second").unwrap();
        let barrier = std::sync::Barrier::new(2);
        let publish = |work: &Path| {
            barrier.wait();
            publish_mp3(work, folder.path())
        };
        let (first_result, second_result) = std::thread::scope(|scope| {
            let first_result = scope.spawn(|| publish(first.path()));
            let second_result = scope.spawn(|| publish(second.path()));
            (first_result.join().unwrap(), second_result.join().unwrap())
        });
        assert_ne!(first_result.is_ok(), second_result.is_ok());
        let expected: &[u8] = if first_result.is_ok() {
            b"first"
        } else {
            b"second"
        };
        assert_eq!(
            std::fs::read(folder.path().join("Song.mp3")).unwrap(),
            expected
        );
        assert!(
            first_result
                .err()
                .or_else(|| second_result.err())
                .unwrap()
                .contains("já existe")
        );
    }

    #[test]
    fn rejects_empty_files_and_symlinks() {
        let folder = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir_in(folder.path()).unwrap();
        std::fs::write(work.path().join("Song.mp3"), b"").unwrap();
        assert!(publish_mp3(work.path(), folder.path()).is_err());
        std::fs::remove_file(work.path().join("Song.mp3")).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/passwd", work.path().join("Song.mp3")).unwrap();
            assert!(publish_mp3(work.path(), folder.path()).is_err());
        }
    }
}
