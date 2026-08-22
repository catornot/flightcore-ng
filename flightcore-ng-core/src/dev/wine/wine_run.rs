use color_eyre::eyre::{Report, eyre};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;
use tracing::info;

use crate::dev::wine::{
    Pipes, run_wine_command,
    wine_install::{install_wine, is_wine_installed, remove_wine},
};

pub async fn run_game(
    exe: &Path,
    launch_args: &[String],
    vanilla: bool,
    debug: bool,
) -> Result<(), Report> {
    if !is_wine_installed() {
        info!("installing wine prefix");

        // todo add progress bar
        if let Err(err) = install_wine().await {
            _ = remove_wine().await;
            return Err(err);
        }
    }

    info!("launching game at {} with {:?}", exe.display(), launch_args);

    // we need -noOriginStartup for maxima
    let mut extra_args = ["-noOriginStartup", "-multiple", "-northstar"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    if vanilla {
        extra_args.push("-vanilla".to_string());
    }

    if debug {
        run_debug(
            exe.to_owned(),
            launch_args
                .iter()
                .chain(extra_args.iter())
                .cloned()
                .collect(),
        )
        .await
    } else {
        run_normally(exe, launch_args.iter().chain(extra_args.iter())).await
    }
}

async fn run_normally(
    exe: &Path,
    launch_args: impl Iterator<Item = &String>,
) -> Result<(), Report> {
    run_wine_command(
        exe,
        launch_args,
        Some(
            exe.parent()
                .ok_or_else(|| eyre!("couldn't find game path for {}", exe.display()))?,
        ),
        None,
    )
    .await
    .map(|_| ())
}

async fn run_debug(exe: PathBuf, mut launch_args: Vec<String>) -> Result<(), Report> {
    let mingw_gdb = String::from_utf8(
        Command::new("nix")
            .args(["path-info", ".#mingw-gdb"])
            .stdin(Stdio::piped())
            .output()
            .await?
            .stdout,
    )?
    .trim()
    .to_string();
    let gdb_exe = mingw_gdb + "/bin/gdb.exe";

    launch_args.push("-waitfordebugger".to_string());

    let launch_args = launch_args
        .get(1..)
        .unwrap_or(&[])
        .iter()
        .fold(launch_args[0].clone(), |acc, arg| acc + "  " + arg);

    run_wine_command(
        &gdb_exe,
        [
            exe.to_string_lossy().as_ref(),
            "-ex",
            &format!("set args {launch_args}"),
            "-ex",
            "run",
            "-ex",
            "c",
        ]
        .into_iter(),
        Some(
            exe.parent()
                .ok_or_else(|| eyre!("couldn't find game path for {}", exe.display()))?,
        ),
        Some(Pipes {
            stdin: Stdio::piped(),
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
        }),
    )
    .await
    // .inspect(|_| _task.abort())
    .map(|_| ())
}
