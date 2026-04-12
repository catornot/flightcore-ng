use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use color_eyre::eyre::Result;
use flightcore_ng_core::dev::wine::{
    wine_install::{install_wine, is_wine_installed, remove_wine},
    wine_run::run_game,
};
use tracing::info;

#[derive(Parser, Debug)]
#[clap(name = "cat_or_not")]
#[clap(author = "@cat_or_not:matrix.catornot.net")]
#[clap(about = "Next Generation tool for Northstar install management")]
#[clap(after_help = "Hi")]
#[command(version, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    ///Show debug output
    #[clap(global = true, short, long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "install-pr")]
    InstallPullRequest {
        #[arg(value_name = "NorthstarMods or NorthstarLauncher url", value_hint = ValueHint::Url)]
        pr: reqwest::Url,

        #[clap(long, short, value_name = "profile", value_hint = ValueHint::Other)]
        profile: Option<String>,
    },

    InstallMod {},

    InstallRepos {},

    #[command(name = "launch")]
    LaunchWine {
        #[arg(value_name = "passthrough args", value_hint = ValueHint::Url)]
        passthrough: Vec<String>,
    },
    // #[command(name = "clean-wine")]
    // CleanWine {},
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Commands::InstallPullRequest { pr, profile } => todo!(),
        Commands::InstallMod {} => todo!(),
        Commands::InstallRepos {} => todo!(),
        Commands::LaunchWine { passthrough } => {
            if !is_wine_installed() {
                info!("installing wine prefix");

                // todo add progress bar
                if let Err(err) = install_wine().await {
                    _ = remove_wine().await;
                    Err(err)?;
                }
            }

            info!(
                "launching the game at /home/catornot/.local/share/Steam/steamapps/common/Titanfall2/NorthstarLauncher.exe"
            );

            run_game(
                &PathBuf::from(
                    "/home/catornot/.local/share/Steam/steamapps/common/Titanfall2/NorthstarLauncher.exe",
                ),
                &passthrough,
            ).await?;
        }
    }

    Ok(())
}
