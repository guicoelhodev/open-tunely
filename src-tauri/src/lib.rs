use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command};
use tauri::Manager;

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
    let binary = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/yt-dlp")
    } else {
        app.path()
            .resource_dir()
            .map_err(|error| format!("Não foi possível localizar as ferramentas: {error}"))?
            .join("binaries/yt-dlp")
    };
    if !binary.is_file() {
        return Err(
            "O yt-dlp do aplicativo não foi preparado. Execute npm run setup:tools no projeto."
                .into(),
        );
    }
    tauri::async_runtime::spawn_blocking(move || search_with_ytdlp(binary, &query))
        .await
        .map_err(|error| format!("A pesquisa foi interrompida: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![search_song])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::parse_song;
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
