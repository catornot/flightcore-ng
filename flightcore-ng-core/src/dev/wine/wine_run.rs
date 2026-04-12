use color_eyre::eyre::{Report, eyre};
use std::path::Path;

use crate::dev::wine::run_wine_command;

pub async fn run_game(exe: &Path, launch_args: &[String]) -> Result<(), Report> {
    run_wine_command(
        exe,
        // we need -noOriginStartup for maxima
        launch_args.iter().chain(&["-noOriginStartup".to_string()]),
        Some(
            exe.parent()
                .ok_or_else(|| eyre!("couldn't find game path for {}", exe.display()))?,
        ),
        None,
    )
    .await
    .map(|_| ())
}
