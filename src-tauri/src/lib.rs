use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::Manager;

mod download;
mod library;

const PREVIEW_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Default)]
struct PreviewState {
    active: Mutex<(u32, Option<Arc<AtomicBool>>)>,
    execution: Mutex<()>,
}

impl PreviewState {
    fn reserve(&self) -> Result<u32, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Não foi possível preparar o áudio.")?;
        let ticket = active
            .0
            .checked_add(1)
            .ok_or("Reinicie o aplicativo para ouvir novamente.")?;
        if let Some(cancelled) = active.1.take() {
            cancelled.store(true, Ordering::SeqCst);
        }
        *active = (ticket, Some(Arc::new(AtomicBool::new(false))));
        Ok(ticket)
    }

    fn token(&self, ticket: u32) -> Result<Arc<AtomicBool>, String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "Não foi possível preparar o áudio.")?;
        if active.0 == ticket {
            if let Some(token) = &active.1 {
                return Ok(Arc::clone(token));
            }
        }
        Err("A preparação do áudio foi cancelada.".into())
    }

    fn cancel(&self, ticket: u32) -> Result<(), String> {
        self.token(ticket)?.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// yt-dlp's standalone build may fork: stop its entire process group on Linux.
struct YtdlpChild {
    child: Child,
    reaped: bool,
}

impl Drop for YtdlpChild {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        #[cfg(unix)]
        unsafe {
            // The child is placed in a new group before spawning. It is not reaped
            // until this guard drops, so its PID cannot be reused during cleanup.
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(unix)]
fn nonblocking_pipe(pipe: &impl std::os::fd::AsRawFd) -> Result<(), String> {
    let fd = pipe.as_raw_fd();
    // These descriptors are owned by the child pipes and remain open here.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
        return Err("Não foi possível preparar a leitura da resposta do yt-dlp.".into());
    }
    Ok(())
}

#[cfg(unix)]
fn read_available(pipe: &mut impl Read, output: &mut Vec<u8>) -> Result<bool, String> {
    let mut buffer = [0; 8192];
    // Bound each pass so continuous output cannot starve cancellation/deadline checks.
    for _ in 0..8 {
        match pipe.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(size) => output.extend_from_slice(&buffer[..size]),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("Não foi possível ler a resposta do yt-dlp.".into()),
        }
    }
    Ok(false)
}

fn run_ytdlp_process(
    command: &mut Command,
    cancelled: &AtomicBool,
    timeout: Duration,
) -> Result<Output, String> {
    run_ytdlp_process_with_progress(command, cancelled, timeout, |_, _| {})
}

fn run_ytdlp_process_with_progress(
    command: &mut Command,
    cancelled: &AtomicBool,
    timeout: Duration,
    mut progress: impl FnMut(&[u8], bool),
) -> Result<Output, String> {
    if cancelled.load(Ordering::SeqCst) {
        return Err("A preparação do áudio foi cancelada.".into());
    }
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = YtdlpChild {
        child: command
            .spawn()
            .map_err(|error| format!("Não foi possível executar o yt-dlp: {error}"))?,
        reaped: false,
    };
    let stdout = child
        .child
        .stdout
        .take()
        .ok_or("Não foi possível ler o áudio.")?;
    let stderr = child
        .child
        .stderr
        .take()
        .ok_or("Não foi possível ler a resposta do yt-dlp.")?;
    #[cfg(unix)]
    {
        nonblocking_pipe(&stdout)?;
        nonblocking_pipe(&stderr)?;
        let (mut stdout_pipe, mut stderr_pipe) = (stdout, stderr);
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let (mut stdout_closed, mut stderr_closed) = (false, false);
        let started = Instant::now();
        loop {
            if cancelled.load(Ordering::SeqCst) {
                return Err("A preparação do áudio foi cancelada.".into());
            }
            if started.elapsed() >= timeout {
                return Err(format!(
                    "A operação demorou mais de {} segundos. Tente novamente.",
                    timeout.as_secs()
                ));
            }
            if !stdout_closed {
                let offset = stdout.len();
                stdout_closed = read_available(&mut stdout_pipe, &mut stdout)?;
                progress(&stdout[offset..], false);
            }
            if !stderr_closed {
                let offset = stderr.len();
                stderr_closed = read_available(&mut stderr_pipe, &mut stderr)?;
                progress(&stderr[offset..], true);
            }
            // Keep the parent unreaped until both pipes close. A descendant may
            // still own them after the parent exits; retaining its PID also makes
            // process-group cleanup safe if cancellation or the deadline fires.
            if stdout_closed && stderr_closed {
                match child.child.try_wait() {
                    Ok(Some(status)) => {
                        child.reaped = true;
                        return Ok(Output {
                            status,
                            stdout,
                            stderr,
                        });
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(format!("Não foi possível acompanhar o yt-dlp: {error}"));
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    #[cfg(not(unix))]
    std::thread::scope(|scope| {
        let read = |mut stream: Box<dyn Read + Send>| {
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes).map(|_| bytes)
        };
        let stdout = scope.spawn(move || read(Box::new(stdout)));
        let stderr = scope.spawn(move || read(Box::new(stderr)));
        let started = Instant::now();
        let status = loop {
            if cancelled.load(Ordering::SeqCst) {
                break Err("A preparação do áudio foi cancelada.".to_string());
            }
            if started.elapsed() >= timeout {
                break Err(format!(
                    "A operação demorou mais de {} segundos. Tente novamente.",
                    timeout.as_secs()
                ));
            }
            match child.child.try_wait() {
                Ok(Some(status)) => {
                    child.reaped = true;
                    break Ok(status);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                Err(error) => break Err(format!("Não foi possível acompanhar o yt-dlp: {error}")),
            }
        };
        // Kill/reap before joining readers, which may be waiting on inherited pipes.
        drop(child);
        let status = status?;
        let stdout = stdout
            .join()
            .map_err(|_| "A leitura do áudio foi interrompida.")?
            .map_err(|_| "Não foi possível ler o áudio.")?;
        let stderr = stderr
            .join()
            .map_err(|_| "A leitura da resposta foi interrompida.")?
            .map_err(|_| "Não foi possível ler a resposta do yt-dlp.")?;
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct Song {
    id: String,
    title: String,
}

fn search_with_ytdlp(binary: PathBuf, query: &str) -> Result<Song, String> {
    let output = Command::new(binary)
        .args([
            "--ignore-config",
            "--no-plugin-dirs",
            "--flat-playlist",
            "--skip-download",
            "--dump-json",
            "--socket-timeout",
            "15",
            "--retries",
            "1",
            "--extractor-retries",
            "1",
            "--",
        ])
        .arg(format!("ytsearch1:{query}"))
        .output()
        .map_err(|error| format!("Não foi possível executar o yt-dlp: {error}"))?;
    if !output.status.success() {
        eprintln!("yt-dlp: {}", String::from_utf8_lossy(&output.stderr));
        return Err(
            "Não foi possível pesquisar no YouTube. Verifique sua conexão e tente novamente."
                .into(),
        );
    }
    parse_song(&output.stdout)
}

fn parse_song(output: &[u8]) -> Result<Song, String> {
    let text = std::str::from_utf8(output)
        .map_err(|_| "O yt-dlp retornou uma resposta inválida.".to_string())?;
    let line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| "Nenhuma música encontrada. Tente outro nome.".to_string())?;
    let song: Song = serde_json::from_str(line)
        .map_err(|_| "O yt-dlp retornou dados inesperados.".to_string())?;
    if song.id.trim().is_empty() || song.title.trim().is_empty() {
        return Err("O resultado não contém um título válido.".into());
    }
    Ok(song)
}

#[tauri::command]
async fn search_song(app: tauri::AppHandle, query: String) -> Result<Song, String> {
    let query = query.trim().to_string();
    if query.is_empty() || query.chars().count() > 300 {
        return Err("Informe uma busca entre 1 e 300 caracteres.".into());
    }
    let binary = tool_binary(&app)?;
    tauri::async_runtime::spawn_blocking(move || search_with_ytdlp(binary, &query))
        .await
        .map_err(|error| format!("A pesquisa foi interrompida: {error}"))?
}

fn tool_binary(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    private_tool(app, "yt-dlp")
}

fn private_tool(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let binary = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(name)
    } else {
        app.path()
            .resource_dir()
            .map_err(|error| format!("Não foi possível localizar as ferramentas: {error}"))?
            .join("binaries")
            .join(name)
    };
    if !binary.is_file() {
        return Err(format!(
            "O {name} do aplicativo não foi preparado. Execute npm run setup:tools no projeto."
        ));
    }
    Ok(binary)
}

fn valid_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn parse_preview_url(output: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(output)
        .map_err(|_| "O yt-dlp retornou um endereço de áudio inválido.".to_string())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let address = lines.next().unwrap_or("").trim();
    let url = tauri::Url::parse(address)
        .map_err(|_| "Não foi possível obter um endereço para ouvir esta música.".to_string())?;
    if url.scheme() != "https" || url.host_str().is_none() || lines.next().is_some() {
        return Err("O yt-dlp retornou um endereço de áudio inválido.".into());
    }
    Ok(address.to_string())
}

fn preview_with_ytdlp(binary: PathBuf, id: &str, cancelled: &AtomicBool) -> Result<String, String> {
    let mut command = Command::new(binary);
    command
        .args([
            "--ignore-config",
            "--no-plugin-dirs",
            "--no-js-runtimes",
            "--no-playlist",
            "--skip-download",
            "--get-url",
            "--format",
            "bestaudio[ext=m4a][protocol=https]/bestaudio[protocol=https]",
            "--socket-timeout",
            "15",
            "--retries",
            "1",
            "--extractor-retries",
            "1",
            "--",
        ])
        .arg(format!("https://www.youtube.com/watch?v={id}"));
    let output = run_ytdlp_process(&mut command, cancelled, PREVIEW_TIMEOUT)?;
    if !output.status.success() {
        eprintln!(
            "yt-dlp preview: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(
            "Não foi possível preparar o áudio. Verifique sua conexão e tente novamente.".into(),
        );
    }
    parse_preview_url(&output.stdout)
}

#[tauri::command]
fn reserve_preview(state: tauri::State<'_, Arc<PreviewState>>) -> Result<u32, String> {
    state.reserve()
}

#[tauri::command]
fn cancel_preview(state: tauri::State<'_, Arc<PreviewState>>, ticket: u32) -> Result<(), String> {
    state.cancel(ticket)
}

#[tauri::command]
async fn preview_song(
    app: tauri::AppHandle,
    id: String,
    ticket: u32,
    state: tauri::State<'_, Arc<PreviewState>>,
) -> Result<String, String> {
    if !valid_video_id(&id) {
        return Err("O identificador da música é inválido.".into());
    }
    let binary = tool_binary(&app)?;
    let cancelled = state.token(ticket)?;
    let state = Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move || {
        // Serialize extraction so the previous child is reaped before starting another.
        let _execution = state
            .execution
            .lock()
            .map_err(|_| "Não foi possível preparar o áudio.")?;
        preview_with_ytdlp(binary, &id, &cancelled)
    })
    .await
    .map_err(|error| format!("A preparação do áudio foi interrompida: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(PreviewState::default()))
        .manage(library::LibraryState::default())
        .manage(Arc::new(download::DownloadState::default()))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            search_song,
            reserve_preview,
            cancel_preview,
            preview_song,
            library::choose_music_folder,
            library::refresh_music_library,
            download::download_song
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{parse_preview_url, parse_song, valid_video_id};

    #[test]
    fn newer_preview_cancels_old_ticket_but_stale_cancel_cannot_stop_new_one() {
        use super::{Ordering, PreviewState};
        let state = PreviewState::default();
        let old_ticket = state.reserve().unwrap();
        let old_token = state.token(old_ticket).unwrap();
        let new_ticket = state.reserve().unwrap();
        let new_token = state.token(new_ticket).unwrap();
        assert!(old_token.load(Ordering::SeqCst));
        assert!(state.cancel(old_ticket).is_err());
        assert!(!new_token.load(Ordering::SeqCst));
        state.cancel(new_ticket).unwrap();
        assert!(new_token.load(Ordering::SeqCst));
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_interrupts_running_process() {
        use super::{AtomicBool, Command, Duration, Instant, Ordering, run_ytdlp_process};
        let cancelled = AtomicBool::new(false);
        let started = Instant::now();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(Duration::from_millis(100));
                cancelled.store(true, Ordering::SeqCst);
            });
            let error = run_ytdlp_process(
                Command::new("sh").args(["-c", "sleep 30 & wait"]),
                &cancelled,
                Duration::from_secs(2),
            )
            .unwrap_err();
            assert!(error.contains("cancelada"));
        });
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn deadline_interrupts_process_with_inherited_pipes() {
        use super::{AtomicBool, Command, Duration, Instant, run_ytdlp_process};
        let started = Instant::now();
        let error = run_ytdlp_process(
            Command::new("sh").args(["-c", "sleep 30 & wait"]),
            &AtomicBool::new(false),
            Duration::from_millis(100),
        )
        .unwrap_err();
        assert!(error.contains("demorou"));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn deadline_still_applies_after_parent_exits_with_either_pipe_open() {
        use super::{AtomicBool, Command, Duration, Instant, run_ytdlp_process};
        for script in [
            "sleep 5 2>/dev/null & exit 0",
            "sleep 5 >/dev/null & exit 0",
        ] {
            let started = Instant::now();
            let result = run_ytdlp_process(
                Command::new("sh").args(["-c", script]),
                &AtomicBool::new(false),
                Duration::from_millis(100),
            );
            assert!(started.elapsed() < Duration::from_secs(2));
            assert!(result.unwrap_err().contains("demorou"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_still_applies_after_parent_exits_with_pipes_open() {
        use super::{AtomicBool, Command, Duration, Instant, Ordering, run_ytdlp_process};
        let cancelled = AtomicBool::new(false);
        let started = Instant::now();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                std::thread::sleep(Duration::from_millis(100));
                cancelled.store(true, Ordering::SeqCst);
            });
            let result = run_ytdlp_process(
                Command::new("sh").args(["-c", "sleep 5 & exit 0"]),
                &cancelled,
                Duration::from_secs(2),
            );
            assert!(started.elapsed() < Duration::from_secs(2));
            assert!(result.unwrap_err().contains("cancelada"));
        });
    }

    #[cfg(unix)]
    #[test]
    fn collects_descendant_output_after_parent_exits_before_deadline() {
        use super::{AtomicBool, Command, Duration, run_ytdlp_process};
        let output = run_ytdlp_process(
            Command::new("sh").args([
                "-c",
                "(sleep 0.1; printf audio; printf diagnostic >&2) & exit 0",
            ]),
            &AtomicBool::new(false),
            Duration::from_secs(2),
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"audio");
        assert_eq!(output.stderr, b"diagnostic");
    }

    #[cfg(unix)]
    #[test]
    fn stopped_child_is_reaped() {
        use super::{Command, YtdlpChild};
        use std::os::unix::process::CommandExt;
        let mut command = Command::new("sh");
        command.args(["-c", "exec sleep 30"]).process_group(0);
        let child = YtdlpChild {
            child: command.spawn().unwrap(),
            reaped: false,
        };
        let pid = child.child.id() as i32;
        drop(child);
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(unix)]
    #[test]
    fn reads_both_pipes_without_deadlock() {
        use super::{AtomicBool, Command, Duration, run_ytdlp_process};
        let output = run_ytdlp_process(
            Command::new("sh").args([
                "-c",
                "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2",
            ]),
            &AtomicBool::new(false),
            Duration::from_secs(2),
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 131072);
        assert_eq!(output.stderr.len(), 131072);
    }
    #[test]
    fn preview_accepts_only_youtube_ids() {
        assert!(valid_video_id("P-lxOj0XpEE"));
        assert!(valid_video_id("abc_DEF-123"));
        for id in [
            "",
            "too-short",
            "https://youtube.com",
            "abcdefghij/",
            "ábcdefghij",
        ] {
            assert!(!valid_video_id(id));
        }
    }
    #[test]
    fn preview_requires_one_https_stream() {
        assert_eq!(
            parse_preview_url(b"\nhttps://example.com/audio?token=abc\n").unwrap(),
            "https://example.com/audio?token=abc"
        );
        for output in [
            "",
            "invalid",
            "http://example.com/audio",
            "file:///tmp/audio",
            "https://a.com\nhttps://b.com",
        ] {
            assert!(parse_preview_url(output.as_bytes()).is_err());
        }
    }
    #[test]
    fn reads_search_metadata_with_unicode_and_extra_fields() {
        let song =
            parse_song(r#"{"id":"abc","title":"Canção — ao vivo","duration":123}"#.as_bytes())
                .unwrap();
        assert_eq!(song.id, "abc");
        assert_eq!(song.title, "Canção — ao vivo");
    }
    #[test]
    fn handles_empty_and_invalid_results() {
        for output in ["", "\n", "not json", "{}", r#"{"id":"abc","title":" "}"#] {
            assert!(parse_song(output.as_bytes()).is_err());
        }
    }
}
