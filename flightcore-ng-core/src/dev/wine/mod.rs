use color_eyre::eyre::{Report, eyre};
use eyre::Context;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;

use crate::local_dir;

pub mod wine_install;
pub mod wine_run;

pub async fn run_wine_command(
    arg: impl AsRef<OsStr>,
    args: impl Iterator<Item = impl AsRef<OsStr>>,
    work_dir: Option<&Path>,
    piped: Option<Stdio>,
) -> Result<String, Report> {
    let proton = proton_dir()?;
    let wine_prefix = wine_dir()?;

    let mut command = Command::new("umu-run");
    command
        .env("UMU_ZENITY", "1")
        .env("WINEPREFIX", wine_prefix)
        .env("PROTONPATH", proton)
        .env("LD_LIBRARY_PATH", "")
        .env("LD_PRELOAD", "")
        .arg(arg)
        .args(args);

    if let Some(work_dir) = work_dir {
        command.current_dir(work_dir);
    }

    let piped = piped.unwrap_or(Stdio::inherit());
    let output = command.stdout(piped).output().await?;

    if !output.status.success() {
        return Err(eyre!(String::from_utf8(output.stderr)?)).wrap_err("couldn't run wine command");
    }

    Ok(String::from_utf8(output.stdout).unwrap_or_default())
}

pub fn proton_dir() -> Result<PathBuf, Report> {
    Ok(local_dir()?.join("proton"))
}

pub fn wine_dir() -> Result<PathBuf, Report> {
    Ok(local_dir()?.join("wine"))
}
