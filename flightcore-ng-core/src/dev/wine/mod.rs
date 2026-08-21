use color_eyre::eyre::{Report, eyre};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;

use crate::local_dir;

pub mod wine_install;
pub mod wine_run;

const WINE_ENV: &[(&str, &str)] = &[
    ("UMU_ZENITY", "1"),
    ("LD_LIBRARY_PATH", ""),
    ("LD_PRELOAD", ""),
    ("STORE", "ea"),
    ("WINEDEBUG", "fixme-all"),
    ("GAMEID", "umu-0"),
    ("PROTON_VERB", "run"),
];

#[derive(Debug)]
pub struct Pipes {
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl Default for Pipes {
    fn default() -> Self {
        Self {
            stdin: Stdio::piped(),
            stdout: Stdio::piped(),
            stderr: Stdio::piped(),
        }
    }
}

pub fn add_wine_command(
    mut command: Command,
    arg: impl AsRef<OsStr>,
    args: impl Iterator<Item = impl AsRef<OsStr>>,
    work_dir: Option<&Path>,
    piped: Option<Pipes>,
) -> Result<Command, Report> {
    let proton = proton_dir()?;
    let wine_prefix = wine_dir()?;
    let piped = piped.unwrap_or_default();

    command
        .arg("umu-run")
        .envs(WINE_ENV.iter().copied())
        .env("WINEPREFIX", wine_prefix)
        .env("PROTONPATH", proton)
        .args(["start.exe", "/b", "/unix"])
        .arg(arg)
        .args(args)
        .stdout(piped.stdout)
        .stderr(piped.stderr)
        .stdin(piped.stdin);

    if let Some(work_dir) = work_dir {
        command.current_dir(work_dir);
    }

    Ok(command)
}

pub async fn run_wine_command(
    arg: impl AsRef<OsStr>,
    args: impl Iterator<Item = impl AsRef<OsStr>>,
    work_dir: Option<&Path>,
    piped: Option<Pipes>,
) -> Result<String, Report> {
    let proton = proton_dir()?;
    let wine_prefix = wine_dir()?;
    let piped = piped.unwrap_or_default();

    let mut command = Command::new("umu-run");
    command
        .envs(WINE_ENV.iter().copied())
        .env("WINEPREFIX", wine_prefix)
        .env("PROTONPATH", proton)
        .arg(arg)
        .args(args);

    if let Some(work_dir) = work_dir {
        command.current_dir(work_dir);
    }

    let command = command
        .stdin(piped.stdin)
        .stdout(piped.stdout)
        .stderr(piped.stderr);

    let mut child = command.spawn()?;

    // print_and_collect_errors(child, "umu-run").await?;
    child.wait().await?;

    Ok("".to_string())
}

pub fn proton_dir() -> Result<PathBuf, Report> {
    Ok(local_dir()?.join("proton"))
}

pub fn wine_dir() -> Result<PathBuf, Report> {
    Ok(local_dir()?.join("wine"))
}
