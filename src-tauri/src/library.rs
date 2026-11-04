use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
pub struct LibraryState(pub Mutex<Option<PathBuf>>);

#[derive(Debug, Serialize)]
pub struct LocalTrack {
    path: String,
    title: String,
    filename: String,
    size: u64,
}

#[derive(Serialize)]
pub struct MusicLibrary {
    folder: String,
    tracks: Vec<LocalTrack>,
}

fn scan_folder(folder: &Path) -> Result<Vec<LocalTrack>, String> {
    let entries = std::fs::read_dir(folder)
        .map_err(|_| "Não foi possível ler a pasta. Verifique as permissões e tente novamente.")?;
    let mut tracks = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| "Não foi possível ler um dos arquivos da pasta.")?;
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
        {
            continue;
        }
        // Only direct, regular files; don't follow links outside the chosen folder.
        if !entry
            .file_type()
            .map_err(|_| "Não foi possível verificar um arquivo da pasta.")?
            .is_file()
        {
            continue;
        }
        let Some(address) = path.to_str() else {
            continue;
        };
        let metadata = entry
            .metadata()
            .map_err(|_| "Não foi possível acessar um dos arquivos MP3.")?;
        tracks.push(LocalTrack {
            path: address.to_owned(),
            title: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            filename: entry.file_name().to_string_lossy().into_owned(),
            size: metadata.len(),
        });
    }
    tracks.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then(a.path.cmp(&b.path))
    });
    Ok(tracks)
}

#[tauri::command]
pub async fn choose_music_folder(app: tauri::AppHandle) -> Result<Option<MusicLibrary>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(folder) = app
            .dialog()
            .file()
            .set_title("Selecionar pasta de músicas")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let folder: PathBuf = folder
            .into_path()
            .map_err(|_| "A pasta selecionada não é um caminho local válido.")?;
        let folder = folder
            .canonicalize()
            .map_err(|_| "Não foi possível acessar a pasta selecionada.")?;
        let library = load_library(&app, &folder)?;
        *app.state::<LibraryState>()
            .0
            .lock()
            .map_err(|_| "Não foi possível guardar a pasta selecionada.")? = Some(folder);
        Ok(Some(library))
    })
    .await
    .map_err(|_| "A seleção da pasta foi interrompida.".to_string())?
}

pub fn selected_folder(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.state::<LibraryState>()
        .0
        .lock()
        .map_err(|_| "Não foi possível acessar a biblioteca.")?
        .clone()
        .ok_or_else(|| "Selecione uma pasta antes de baixar.".into())
}

fn load_library(app: &tauri::AppHandle, folder: &Path) -> Result<MusicLibrary, String> {
    let tracks = scan_folder(folder)?;
    let scope = app.asset_protocol_scope();
    for track in &tracks {
        scope
            .allow_file(&track.path)
            .map_err(|_| "Não foi possível liberar a reprodução dos arquivos MP3.")?;
    }
    Ok(MusicLibrary {
        folder: folder.to_string_lossy().into_owned(),
        tracks,
    })
}

#[tauri::command]
pub async fn refresh_music_library(app: tauri::AppHandle) -> Result<Option<MusicLibrary>, String> {
    let folder = app
        .state::<LibraryState>()
        .0
        .lock()
        .map_err(|_| "Não foi possível acessar a biblioteca.")?
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        folder.map(|folder| load_library(&app, &folder)).transpose()
    })
    .await
    .map_err(|_| "A atualização da biblioteca foi interrompida.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::scan_folder;

    #[test]
    fn lists_only_direct_mp3_files_sorted_and_keeps_original_paths() {
        let folder = std::env::temp_dir().join(format!(
            "mp3-library-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(folder.join("nested.mp3")).unwrap();
        for filename in [
            "Zebra.MP3",
            "Áudio.mp3",
            "alpha.mp3",
            "ignore.wav",
            "ignore.mp3.txt",
        ] {
            std::fs::write(folder.join(filename), b"sample").unwrap();
        }
        std::fs::write(folder.join("nested.mp3/hidden.mp3"), b"sample").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(folder.join("alpha.mp3"), folder.join("link.mp3")).unwrap();
        let tracks = scan_folder(&folder).unwrap();
        assert_eq!(
            tracks
                .iter()
                .map(|track| track.title.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "Zebra", "Áudio"]
        );
        assert_eq!(tracks[0].path, folder.join("alpha.mp3").to_str().unwrap());
        assert_eq!(tracks[0].size, 6);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn missing_folder_returns_actionable_error() {
        assert!(
            scan_folder(&std::env::temp_dir().join("missing-mp3-library-parent/absent"))
                .unwrap_err()
                .contains("permissões")
        );
    }
}
