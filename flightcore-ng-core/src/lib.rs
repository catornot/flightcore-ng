use color_eyre::eyre::{Context, Report, eyre};
use futures_lite::StreamExt;
use std::path::{Path, PathBuf};
use tokio::fs;

pub mod dev;
pub mod launch;
pub mod settings;
pub mod setup;

pub const TITANFALL_ID: u32 = 1237970;

pub fn local_dir() -> Result<PathBuf, color_eyre::Report> {
    let dirs = directories::ProjectDirs::from("org", "flightcore", "flightcore-ng");
    let path = dirs.unwrap().data_local_dir().to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn tmp_dir() -> Result<PathBuf, color_eyre::Report> {
    Ok(local_dir()?.join("tmp"))
}

// TODO: move this

pub async fn create_backup(file_path: &Path, delete: bool) -> Result<(), Report> {
    let file_parent = file_path
        .parent()
        .ok_or_else(|| eyre!("couldn't find parent for {}", file_path.display()))?;
    let mut entries = async_walkdir::WalkDir::new(file_parent);

    let mut max_backup = 0u32;
    let file_name = file_path
        .file_name()
        .ok_or_else(|| eyre!("couldn't find file name for {}", file_path.display()))?
        .to_string_lossy();
    while let Some(path) = entries.next().await {
        let Ok(path) = path.map(|path| path.path()) else {
            continue;
        };

        if let Some(backup) = path
            .file_name()
            .filter(|name| name.to_string_lossy().starts_with(&file_name[..]))
            .is_none()
            .then(|| {
                path.extension()
                    .map(|extension| extension.to_string_lossy())
                    .filter(|extension| extension.starts_with("bak"))
                    .and_then(|extension| {
                        extension
                            .split_once("bak")
                            .map(|(_, right)| right)
                            .and_then(|backup| backup.parse::<u32>().ok())
                    })
            })
            .flatten()
        {
            max_backup = max_backup.max(backup);
        }
    }

    let backup_path = file_parent.join(file_name.to_string() + ".bak" + &max_backup.to_string());
    if backup_path.exists() {
        return Err(eyre!(
            "somehow a backup was attempt to be made that already existed : {backup_path:?}"
        ));
    }

    fs::copy(file_path, backup_path)
        .await
        .map(|_| ())
        .wrap_err("failed to copy original to backup")?;

    if delete {
        fs::remove_file(file_path)
            .await
            .wrap_err("failed to remove original file")
    } else {
        Ok(())
    }
}
