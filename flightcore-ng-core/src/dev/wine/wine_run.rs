use color_eyre::eyre::{Report, eyre};
use eyre::Context;
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
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
    let mut extra_args = Vec::from_iter(
        ["-noOriginStartup", "-multiple", "-northstar"]
            .into_iter()
            .map(String::from),
    );
    if vanilla {
        extra_args.push("-vanilla".to_string());
    }

    if !debug {
        run_normally(exe, launch_args.iter().chain(extra_args.iter())).await
    } else {
        run_debug(
            exe.to_owned(),
            launch_args
                .iter()
                .chain(extra_args.iter())
                .cloned()
                .collect(),
        )
        .await
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
    const DEBUG_ADDRESS: &str = "localhost:12345";

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
    let gdb_server_exe = mingw_gdb.clone() + "/bin/gdbserver.exe";
    let gdb_exe = mingw_gdb + "/bin/gdb.exe";

    launch_args.push("-waitfordebugger".to_string());

    // WARN: assumes alacritty for now
    let terminal = env::var("TERM").wrap_err_with(|| {
        eyre!("searched $TERM for the preferred terminal and couldn't find it")
    })?;

    // let _task = tokio::task::spawn(async move {
    //     let mut term = Command::new(terminal);
    //     term.arg("-e");

    //     let attach_remote_command = format!("target remote {DEBUG_ADDRESS}");
    //     let mut term = add_wine_command(
    //         term,
    //         gdb_exe,
    //         ["-ex", &attach_remote_command, "-ex", "c"].into_iter(),
    //         None,
    //         None,
    //     )?;

    //     let child = term.spawn()?;
    //     info!("{:?}", print_and_collect_errors(child, "gdb.exe").await);

    //     Ok::<_, Report>(())
    // });

    let launch_args = launch_args
        .get(1..)
        .unwrap_or(&[])
        .iter()
        .fold(launch_args[0].to_string(), |acc, arg| acc + "  " + arg);

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

// async fn run_debug(exe: PathBuf, mut launch_args: Vec<String>) -> Result<(), Report> {
//     launch_args.push("-waitfordebugger".to_string());
//     let exe_full = exe.to_string_lossy();
//     let exe_name = exe_full.split("/").last().unwrap_or(exe_full.as_ref());

//     let exe_cloned = exe.clone();
//     let game_task =
//         tokio::task::spawn(async move {
//             run_wine_command(
//                 &exe_cloned,
//                 launch_args.iter(),
//                 Some(exe_cloned.parent().ok_or_else(|| {
//                     eyre!("couldn't find game path for {}", exe_cloned.display())
//                 })?),
//                 None,
//             )
//             .await
//             .map(|_| ())
//         });

//     info!("looking for titanfall 2 pid : {exe_name}");
//     let mut found_pid = None::<u32>;
//     loop {
//         let system = System::new_all();
//         for (pid, proc) in system.processes() {
//             if proc.exe().contains(exe_name) {
//                 found_pid.replace(pid.as_u32());
//                 break;
//             }
//         }

//         if found_pid.is_some() || game_task.is_finished() {
//             break;
//         }

//         sleep(Duration::from_millis(50)).await;
//     }

//     if let Some(pid) = found_pid {
//         info!("found it; spawning debugger");
//         run_wine_command(
//             "winedbg",
//             [pid.to_string()].iter(),
//             Some(
//                 exe.parent()
//                     .ok_or_else(|| eyre!("couldn't find game path for {}", exe.display()))?,
//             ),
//             None,
//         )
//         .await?;
//     }

//     game_task.await?
// }

// async fn print_and_collect_errors(
//     mut child: tokio::process::Child,
//     command: &str,
// ) -> Result<(), Report> {
//     let stdout = child
//         .stdout
//         .take()
//         .ok_or_else(|| eyre!("couldn't capture stdout for {command}"))?;
//     let stderr = child
//         .stderr
//         .take()
//         .ok_or_else(|| eyre!("couldn't capture stdout for {command}"))?;

//     let stdout_handle = tokio::spawn(async move {
//         let mut reader = BufReader::new(stdout).lines();

//         while let Some(line) = reader.next_line().await.ok().flatten() {
//             info!("{}", line);
//         }
//     });

//     let stderr_handle = tokio::spawn(async move {
//         let mut reader = BufReader::new(stderr).lines();

//         while let Some(line) = reader.next_line().await.ok().flatten() {
//             info!("{}", line);
//         }
//     });

//     child.wait().await?;
//     stdout_handle.await?;
//     stderr_handle.await?;

//     Ok(())
// }
