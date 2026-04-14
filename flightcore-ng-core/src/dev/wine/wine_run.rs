use color_eyre::eyre::{Report, eyre};
use std::path::Path;
use tracing::info;

use crate::dev::wine::{
    run_wine_command,
    wine_install::{install_wine, is_wine_installed, remove_wine},
};

pub async fn run_game(exe: &Path, launch_args: &[String], vanilla: bool) -> Result<(), Report> {
    if !is_wine_installed() {
        info!("installing wine prefix");

        // todo add progress bar
        if let Err(err) = install_wine().await {
            _ = remove_wine().await;
            return Err(err);
        }
    }

    info!("launching game at {}", exe.display());

    // we need -noOriginStartup for maxima
    let mut extra_args = Vec::from_iter(
        ["-noOriginStartup", "-multiple", "-northstar"]
            .into_iter()
            .map(String::from),
    );
    if vanilla {
        extra_args.push("-vanilla".to_string());
    }

    run_wine_command(
        exe,
        launch_args.iter().chain(extra_args.iter()),
        Some(
            exe.parent()
                .ok_or_else(|| eyre!("couldn't find game path for {}", exe.display()))?,
        ),
        None,
    )
    .await
    .map(|_| ())
}
