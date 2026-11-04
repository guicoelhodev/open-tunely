use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::Manager;

#[derive(Default)]
pub struct AudioHttpState(Mutex<Option<Server>>);
struct Server {
    port: u16,
    tracks: Arc<Mutex<HashMap<String, String>>>,
}

#[tauri::command]
pub fn local_audio_url(app: tauri::AppHandle, path: String) -> Result<String, String> {
    crate::library::checked_track(&crate::library::selected_folder(&app)?, &path)?;
    let state = app.state::<AudioHttpState>();
    let mut server = state.0.lock().map_err(|_| "Player indisponível.")?;
    if server.is_none() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|_| "Não foi possível iniciar o player HTTP.")?;
        let port = listener
            .local_addr()
            .map_err(|_| "Player indisponível.")?
            .port();
        let tracks = Arc::new(Mutex::new(HashMap::new()));
        for _ in 0..4 {
            let listener = listener.try_clone().map_err(|_| "Player indisponível.")?;
            let tracks = tracks.clone();
            let app = app.clone();
            std::thread::spawn(move || {
                for mut stream in listener.incoming().flatten() {
                    let _ = serve(&mut stream, &tracks, &app);
                }
            });
        }
        *server = Some(Server { port, tracks });
    }
    let server = server.as_ref().unwrap();
    // tempfile uses OS randomness; the random name is an unguessable capability.
    let temporary = tempfile::Builder::new()
        .prefix("")
        .rand_bytes(32)
        .tempfile()
        .map_err(|_| "Não foi possível preparar o áudio.")?;
    let token = temporary
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    server
        .tracks
        .lock()
        .map_err(|_| "Player indisponível.")?
        .insert(token.clone(), path);
    Ok(format!("http://127.0.0.1:{}/{token}", server.port))
}

#[tauri::command]
pub fn release_local_audio(state: tauri::State<'_, AudioHttpState>, url: String) {
    if let Ok(server) = state.0.lock() {
        if let Some(server) = server.as_ref() {
            let prefix = format!("http://127.0.0.1:{}/", server.port);
            if let Some(token) = url.strip_prefix(&prefix) {
                if let Ok(mut tracks) = server.tracks.lock() {
                    tracks.remove(token);
                }
            }
        }
    }
}

fn byte_range(header: Option<&str>, size: u64) -> Result<(u64, u64), ()> {
    if size == 0 {
        return Err(());
    }
    let Some(header) = header else {
        return Ok((0, size - 1));
    };
    let (start, end) = header
        .strip_prefix("bytes=")
        .ok_or(())?
        .split_once('-')
        .ok_or(())?;
    if start.is_empty() {
        let suffix: u64 = end.parse().map_err(|_| ())?;
        if suffix == 0 {
            return Err(());
        }
        return Ok((size.saturating_sub(suffix), size - 1));
    }
    let start: u64 = start.parse().map_err(|_| ())?;
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse::<u64>().map_err(|_| ())?.min(size - 1)
    };
    if start >= size || end < start {
        return Err(());
    }
    Ok((start, end))
}

fn serve(
    stream: &mut TcpStream,
    tracks: &Mutex<HashMap<String, String>>,
    app: &tauri::AppHandle,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new((&mut *stream).take(8192));
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let request: Vec<_> = line.split_whitespace().map(str::to_owned).collect();
    let mut range = None;
    let mut complete = false;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line == "\r\n" {
            complete = true;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("range") {
                range = Some(value.trim().to_owned());
            }
        }
    }
    drop(reader);
    if !complete || request.len() != 3 || !matches!(request[0].as_str(), "GET" | "HEAD") {
        return stream.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
    }
    let path = tracks
        .lock()
        .ok()
        .and_then(|tracks| tracks.get(request[1].trim_start_matches('/')).cloned());
    let file = path.and_then(|path| {
        let folder = crate::library::selected_folder(app).ok()?;
        let path = crate::library::checked_track(&folder, &path).ok()?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        options.open(path).ok()
    });
    let Some(mut file): Option<File> = file else {
        return stream.write_all(
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
    };
    write_audio(stream, &mut file, &request[0], range.as_deref())
}

fn write_audio(
    stream: &mut impl Write,
    file: &mut File,
    method: &str,
    range: Option<&str>,
) -> std::io::Result<()> {
    let size = file.metadata()?.len();
    let Ok((start, end)) = byte_range(range, size) else {
        return write!(
            stream,
            "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{size}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
    };
    let length = end - start + 1;
    write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: audio/mpeg\r\nContent-Length: {length}\r\nAccept-Ranges: bytes\r\nCache-Control: no-store\r\nConnection: close\r\n",
        if range.is_some() {
            "206 Partial Content"
        } else {
            "200 OK"
        }
    )?;
    if range.is_some() {
        write!(stream, "Content-Range: bytes {start}-{end}/{size}\r\n")?;
    }
    stream.write_all(b"\r\n")?;
    if method == "GET" {
        file.seek(SeekFrom::Start(start))?;
        std::io::copy(&mut file.take(length), stream)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{byte_range, write_audio};
    #[test]
    fn streams_exact_ranges_and_head_without_body() {
        use std::io::Write;
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(b"0123456789").unwrap();
        for (method, range, status, body) in [
            ("GET", None, "200 OK", "0123456789"),
            ("GET", Some("bytes=3-5"), "206 Partial Content", "345"),
            ("GET", Some("bytes=-2"), "206 Partial Content", "89"),
            ("HEAD", None, "200 OK", ""),
            ("GET", Some("bytes=20-"), "416 Range Not Satisfiable", ""),
        ] {
            let mut response = Vec::new();
            write_audio(&mut response, &mut file, method, range).unwrap();
            let response = String::from_utf8(response).unwrap();
            let (headers, actual) = response.split_once("\r\n\r\n").unwrap();
            assert!(headers.starts_with(&format!("HTTP/1.1 {status}")));
            assert_eq!(actual, body);
            if range == Some("bytes=3-5") {
                assert!(headers.contains("Content-Range: bytes 3-5/10"));
            }
        }
    }

    #[test]
    fn handles_full_partial_suffix_and_invalid_ranges() {
        assert_eq!(byte_range(None, 10), Ok((0, 9)));
        assert_eq!(byte_range(Some("bytes=0-0"), 10), Ok((0, 0)));
        assert_eq!(byte_range(Some("bytes=4-"), 10), Ok((4, 9)));
        assert_eq!(byte_range(Some("bytes=-3"), 10), Ok((7, 9)));
        assert_eq!(byte_range(Some("bytes=3-99"), 10), Ok((3, 9)));
        for header in ["bytes=10-", "bytes=5-4", "bytes=-0", "bytes=0-1,4-5", "bad"] {
            assert!(byte_range(Some(header), 10).is_err());
        }
        assert!(byte_range(None, 0).is_err());
    }
}
