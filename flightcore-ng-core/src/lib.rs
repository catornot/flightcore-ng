use std::{fs, path::PathBuf};

pub mod dev;

pub fn local_dir() -> Result<PathBuf, color_eyre::Report> {
    let dirs = directories::ProjectDirs::from("org", "flightcore", "flightcore-ng");
    let path = dirs.unwrap().data_local_dir().to_path_buf();
    fs::create_dir_all(&path)?;
    Ok(path)
}
