use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

pub struct LibraryState(pub Mutex<Option<PathBuf>>);

impl Default for LibraryState {
    fn default() -> Self {
        Self(Mutex::new(development_folder()))
    }
}

fn development_folder() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        #[derive(serde::Deserialize)]
        struct LocalConfig {
            music_folder: PathBuf,
        }
        let config = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dev-library.local.json");
        let bytes = match std::fs::read(config) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => {
                eprintln!("Não foi possível ler a configuração local da biblioteca: {error}");
                return None;
            }
        };
        let folder = serde_json::from_slice::<LocalConfig>(&bytes)
            .ok()
            .and_then(|config| config.music_folder.canonicalize().ok())
            .filter(|folder| folder.is_dir());
        if folder.is_none() {
            eprintln!("Configuração local da biblioteca inválida ou pasta inacessível.");
        }
        folder
    }
    #[cfg(not(debug_assertions))]
    None
}

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
        .map_err(|_| "ERR_READ_PERMISSION|Não foi possível ler a pasta. Verifique as permissões e tente novamente.")?;
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
pub async fn choose_music_folder(app: tauri::AppHandle, title: String) -> Result<Option<MusicLibrary>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(folder) = app
            .dialog()
            .file()
            .set_title(title)
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

fn resolve_saved_folder(path: &str) -> Result<PathBuf, String> {
    let folder = PathBuf::from(path).canonicalize().map_err(|_| {
        "ERR_SAVED_FOLDER_UNAVAILABLE|Não foi possível acessar a pasta salva. Selecione outra pasta.".to_string()
    })?;
    if !folder.is_dir() {
        return Err("ERR_SAVED_FOLDER_UNAVAILABLE|Não foi possível acessar a pasta salva. Selecione outra pasta.".into());
    }
    Ok(folder)
}

#[tauri::command]
pub async fn restore_music_folder(app: tauri::AppHandle, path: String) -> Result<MusicLibrary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Never keep an unrelated development folder active if restoration fails.
        *app.state::<LibraryState>().0.lock()
            .map_err(|_| "Não foi possível acessar a biblioteca.")? = None;
        let folder = resolve_saved_folder(&path)?;
        let library = load_library(&app, &folder)?;
        *app.state::<LibraryState>().0.lock()
            .map_err(|_| "Não foi possível guardar a pasta selecionada.")? = Some(folder);
        Ok(library)
    }).await.map_err(|_| "A restauração da pasta foi interrompida.".to_string())?
}

pub fn selected_folder(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.state::<LibraryState>()
        .0
        .lock()
        .map_err(|_| "Não foi possível acessar a biblioteca.")?
        .clone()
        .ok_or_else(|| "ERR_FOLDER_MISSING|Selecione uma pasta antes de baixar.".into())
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

pub(crate) fn checked_track(folder: &Path, address: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(address);
    if path.parent() != Some(folder)
        || !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"))
        || !std::fs::symlink_metadata(&path).is_ok_and(|meta| meta.file_type().is_file())
    {
        return Err("ERR_FILE_MISSING|O MP3 não está mais disponível na pasta selecionada.".into());
    }
    Ok(path)
}

fn renamed_path(path: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    let stem = if name.to_ascii_lowercase().ends_with(".mp3") {
        &name[..name.len() - 4]
    } else {
        name
    };
    if stem.is_empty()
        || stem == "."
        || stem == ".."
        || stem.len() > 240
        || stem
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
        || stem.ends_with([' ', '.'])
    {
        return Err(
            "ERR_NAME_INVALID|Informe um nome válido, sem barras ou caracteres especiais de arquivo.".into(),
        );
    }
    Ok(path.with_file_name(format!("{stem}.mp3")))
}

fn rename_track_file(path: &Path, destination: &Path) -> Result<(), String> {
    if path == destination {
        return Ok(());
    }
    // Atomic no-replace rename: an existing song must never be overwritten.
    #[cfg(target_os = "linux")]
    let result = {
        use std::os::unix::ffi::OsStrExt;
        let from =
            std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_| "Caminho inválido.")?;
        let to = std::ffi::CString::new(destination.as_os_str().as_bytes())
            .map_err(|_| "Caminho inválido.")?;
        // SAFETY: both C strings stay alive throughout the call and contain no NUL bytes.
        let code = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if code == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    };
    #[cfg(target_os = "windows")]
    let result = {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileW(existing: *const u16, new: *const u16) -> i32;
        }
        let from: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let to: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
        // MoveFileW fails if the destination exists; std::fs::rename replaces it.
        if unsafe { MoveFileW(from.as_ptr(), to.as_ptr()) } != 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    };
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let result: std::io::Result<()> = Err(std::io::ErrorKind::Unsupported.into());
    result.map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "ERR_NAME_EXISTS|Já existe um arquivo com esse nome. Escolha outro nome.".into()
        } else {
            "ERR_RENAME_PERMISSION|Não foi possível renomear o MP3. Verifique as permissões da pasta.".into()
        }
    })
}

#[tauri::command]
pub async fn rename_music_track(
    app: tauri::AppHandle,
    path: String,
    name: String,
) -> Result<(), String> {
    let folder = selected_folder(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = checked_track(&folder, &path)?;
        let destination = renamed_path(&path, &name)?;
        rename_track_file(&path, &destination)
    })
    .await
    .map_err(|_| "A alteração do nome foi interrompida.".to_string())?
}

#[tauri::command]
pub async fn delete_music_track(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let folder = selected_folder(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = checked_track(&folder, &path)?;
        std::fs::remove_file(path).map_err(|_| {
            "ERR_DELETE_PERMISSION|Não foi possível excluir o MP3. Verifique as permissões da pasta.".to_string()
        })
    })
    .await
    .map_err(|_| "A exclusão foi interrompida.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::{checked_track, rename_track_file, renamed_path, resolve_saved_folder, scan_folder};

    #[test]
    fn mutations_only_accept_direct_regular_mp3_files() {
        let folder = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let track = folder.path().join("song.mp3");
        std::fs::write(&track, b"song").unwrap();
        assert_eq!(
            checked_track(folder.path(), track.to_str().unwrap()).unwrap(),
            track
        );
        for filename in ["other.wav", "directory.mp3"] {
            let path = folder.path().join(filename);
            if filename.ends_with("wav") {
                std::fs::write(&path, b"wave").unwrap();
            } else {
                std::fs::create_dir(&path).unwrap();
            }
            assert!(checked_track(folder.path(), path.to_str().unwrap()).is_err());
        }
        let external = outside.path().join("song.mp3");
        std::fs::write(&external, b"external").unwrap();
        assert!(checked_track(folder.path(), external.to_str().unwrap()).is_err());
        #[cfg(unix)]
        {
            let link = folder.path().join("link.mp3");
            std::os::unix::fs::symlink(&external, &link).unwrap();
            assert!(checked_track(folder.path(), link.to_str().unwrap()).is_err());
        }
    }

    #[test]
    fn rename_validates_names_and_preserves_mp3_extension() {
        let path = Path::new("/library/song.mp3");
        for name in [
            "",
            " ",
            ".",
            "..",
            ".mp3",
            "../escape",
            "sub/song",
            "sub\\song",
            "bad\nname",
            "trailing.",
        ] {
            assert!(renamed_path(path, name).is_err(), "{name:?}");
        }
        assert_eq!(
            renamed_path(path, " Novo nome ").unwrap(),
            Path::new("/library/Novo nome.mp3")
        );
        assert_eq!(
            renamed_path(path, "Novo nome.MP3").unwrap(),
            Path::new("/library/Novo nome.mp3")
        );
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn rename_never_overwrites_an_existing_file() {
        let folder = tempfile::tempdir().unwrap();
        let source = folder.path().join("source.mp3");
        let destination = folder.path().join("destination.mp3");
        std::fs::write(&source, b"source").unwrap();
        std::fs::write(&destination, b"existing").unwrap();
        assert!(rename_track_file(&source, &destination).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), b"source");
        assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
        std::fs::remove_file(&destination).unwrap();
        rename_track_file(&source, &destination).unwrap();
        assert!(!source.exists());
        assert_eq!(std::fs::read(destination).unwrap(), b"source");
    }

    use std::path::Path;

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
    fn saved_folder_must_be_an_existing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("music");
        std::fs::create_dir(&path).unwrap();
        assert_eq!(resolve_saved_folder(path.to_str().unwrap()).unwrap(), path.canonicalize().unwrap());
        let file = dir.path().join("song.mp3");
        std::fs::write(&file, b"audio").unwrap();
        assert!(resolve_saved_folder(file.to_str().unwrap()).unwrap_err().starts_with("ERR_SAVED_FOLDER_UNAVAILABLE|"));
        assert!(resolve_saved_folder(dir.path().join("missing").to_str().unwrap()).unwrap_err().starts_with("ERR_SAVED_FOLDER_UNAVAILABLE|"));
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
