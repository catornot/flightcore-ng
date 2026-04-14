use clap::{Parser, Subcommand, ValueHint};
use color_eyre::eyre::Result;
use eyre::eyre;
use flightcore_ng_core::{
    TITANFALL_ID,
    dev::{
        fetch_revs::fetch_latest,
        install_northstar::{NorthstarInstallInfo, get_northstar_from_revs},
        wine::wine_run::run_game,
    },
    install_northstar,
    settings::FlightCoreSettings,
};
use inquire::validator::{ErrorMessage, Validation};
use std::path::PathBuf;
use steamlocate::SteamDir;
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

        #[clap(long, short, value_name = "profile", value_hint = ValueHint::Other)]
        profile: Option<String>,
    },
    // #[command(name = "clean-wine")]
    // CleanWine {},
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut settings = FlightCoreSettings::load().await?;

    let profile = match &args.command {
        Commands::InstallPullRequest { pr: _, profile }
        | Commands::LaunchWine {
            passthrough: _,
            profile,
        } if profile.is_some() => profile.as_ref().unwrap().as_str(),
        Commands::InstallMod {}
        | Commands::InstallRepos {}
        | Commands::InstallPullRequest { pr: _, profile: _ }
        | Commands::LaunchWine {
            passthrough: _,
            profile: _,
        } => "R2Northstar",
    };

    let titanfall_path = settings
        .get_titanfall_path_from_profile(profile)
        .or_else(|| settings.get_default_titanfall_path())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| ask_titanfall_path(&mut settings));

    match args.command {
        Commands::InstallPullRequest { pr, profile } => {
            let (launcher, mods) = if pr
                .path()
                .split('/')
                .nth(2)
                .filter(|repo| *repo == "NorthstarLauncher")
                .is_some()
            {
                (
                    fetch_latest(pr).await?.sha,
                    fetch_latest("https://github.com/R2Northstar/NorthstarMods".try_into()?)
                        .await?
                        .sha,
                )
            } else if pr
                .path()
                .split('/')
                .nth(2)
                .filter(|repo| *repo == "NorthstarMods")
                .is_some()
            {
                (
                    fetch_latest("https://github.com/R2Northstar/NorthstarLauncher".try_into()?)
                        .await?
                        .sha,
                    fetch_latest(pr).await?.sha,
                )
            } else {
                return Err(eyre!("tried to use a foreign repo"))?;
            };

            let profile = profile.as_deref().unwrap_or("R2NorthstarDev");
            install_northstar(
                &get_northstar_from_revs(NorthstarInstallInfo::new(mods, launcher)).await?,
                profile,
                settings
                    .get_titanfall_path_from_profile(profile)
                    .unwrap_or(&titanfall_path),
            )
            .await?;
        }
        Commands::InstallMod {} => todo!(),
        Commands::InstallRepos {} => todo!(),
        Commands::LaunchWine {
            mut passthrough,
            profile,
        } => {
            if let Some(profile) = profile.as_ref() {
                info!("using profile {profile}");
                passthrough.push(format!("-profile={profile}"));
            }
            run_game(
                &titanfall_path.join("NorthstarLauncher.exe"),
                &passthrough,
                false,
            )
            .await?;
        }
    }

    Ok(())
}

fn ask_titanfall_path(settings: &mut FlightCoreSettings) -> PathBuf {
    let path = SteamDir::locate().ok().and_then(|steam_dir| {
        let (app, library) = steam_dir.find_app(TITANFALL_ID).ok()??;

        Some(library.resolve_app_dir(&app))
    });

    let selection = inquire::Select::new(
        "Select a titanfall 2 path",
        path.map(|path| vec![path.display().to_string(), "manual".to_string()])
            .unwrap_or_else(|| vec!["manual".to_string()]),
    )
    .with_page_size(5)
    .prompt();

    let titanfall = match selection.as_ref().map(|s| s.as_str()) {
        Err(_) | Ok("manual") => inquire::Text::new("Manually enter the path")
            .with_validator(|path: &str| {
                let validation_path = PathBuf::from(path).join("Titanfall2.exe");
                if !validation_path.exists() {
                    Ok(Validation::Invalid(ErrorMessage::Custom(
                        "Not a valid titanfall 2 directory!".to_string(),
                    )))
                } else if !validation_path.is_absolute() {
                    Ok(Validation::Invalid(ErrorMessage::Custom(
                        "Not an absolute path!".to_string(),
                    )))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
            .map(PathBuf::from)
            .expect("couldn't get path for titanfall 2"),
        Ok(path) => PathBuf::from(path),
    };

    settings.add_titanfall_path(titanfall.clone());

    titanfall
}
