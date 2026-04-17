use clap::{Parser, Subcommand, ValueHint};
use color_eyre::eyre::Result;
use eyre::{Context, eyre};
use flightcore_ng_core::{
    TITANFALL_ID, create_backup,
    dev::fetch_revs::fetch_latest,
    launch::launch_northstar,
    settings::{
        CoreModsSource, DiscordRPCSource, FlightCoreSettings, LauncherSource, NorthstarSource,
        SETTINGS_PATH, Source,
    },
    setup::northstar::bootstrap_northstar,
};
use inquire::validator::{ErrorMessage, Validation};
use std::path::PathBuf;
use steamlocate::SteamDir;
use tracing::{error, info, warn};

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
    #[command(name = "install-pr", alias = "pr")]
    InstallPullRequest {
        #[arg(value_name = "NorthstarMods or NorthstarLauncher url", value_hint = ValueHint::Url)]
        #[arg(
            help = "Specify a pr url for any of the repos that are part of the northstar distribution! any amount of prs up to 3 for each repo of course!"
        )]
        urls: Vec<reqwest::Url>,

        #[clap(long, short, value_name = "profile", value_hint = ValueHint::Other)]
        profile: Option<String>,

        #[clap(long, value_name = "force install pr")]
        #[arg(
            help = "force install commit based northstar into this profile; it will permanently modify this profile to be based on commits!"
        )]
        force: bool,
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

    #[command(name = "edit")]
    Edit {
        #[arg(value_name = "editor path", value_hint = ValueHint::AnyPath)]
        editor: Option<String>,
    },

    #[command(name = "open")]
    Open {
        #[arg(long, short, value_name = "the profile which should be opened", value_hint = ValueHint::AnyPath)]
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

    let mut settings = match FlightCoreSettings::load().await {
        Err(_) => {
            warn!("made a backup for settings");
            warn!("settings reset since they couldn't be loaded");
            create_backup(SETTINGS_PATH.as_path(), true).await?;
            FlightCoreSettings::load().await?
        }
        Ok(settings) => settings,
    };

    let profile = match &args.command {
        Commands::InstallPullRequest {
            urls: _,
            profile,
            force: _,
        }
        | Commands::LaunchWine {
            passthrough: _,
            profile,
        }
        | Commands::Open { profile }
            if profile.is_some() =>
        {
            profile.as_ref().unwrap().as_str()
        }
        Commands::InstallMod {}
        | Commands::InstallRepos {}
        | Commands::InstallPullRequest {
            urls: _,
            profile: _,
            force: _,
        }
        | Commands::Edit { editor: _ }
        | Commands::LaunchWine {
            passthrough: _,
            profile: _,
        }
        | Commands::Open { profile: _ } => "R2Northstar",
    };

    let titanfall_path = settings
        .get_titanfall_path_from_profile(profile)
        .or_else(|| settings.get_default_titanfall_path())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| ask_titanfall_path(&mut settings));

    match args.command {
        Commands::InstallPullRequest {
            urls,
            profile,
            force,
        } => {
            let repo_iter = urls.iter().filter_map(|url| url.path().split('/').nth(2));
            let launcher = repo_iter
                .clone()
                .position(|repo| repo == "NorthstarLauncher")
                .and_then(|index| urls.get(index))
                .cloned();
            let mods = repo_iter
                .clone()
                .position(|repo| repo == "NorthstarMods")
                .and_then(|index| urls.get(index))
                .cloned();
            let discord_rpc = repo_iter
                .clone()
                .position(|repo| repo == "NorthstarDiscordRPC")
                .and_then(|index| urls.get(index))
                .cloned();

            if launcher.is_none() && mods.is_none() && discord_rpc.is_none() {
                Err(eyre!("tried to use a foreign repo"))?;
            }

            let profile = profile.as_deref().unwrap_or("R2NorthstarDev");
            let profile = match settings.get_profile_mut(profile) {
                Some(profile) => profile,
                None => {
                    let profile = settings
                        .add_profile(profile, Some(titanfall_path))
                        .expect("this invariant should have been upheld");
                    profile.northstar = NorthstarSource::Overlayed; // set overlay-ed for new profiles
                    profile
                }
            };

            if !matches!(profile.northstar, NorthstarSource::Overlayed) && !force {
                error!("this is profile isn't setup for commit based installations!");
                error!("you can override that by using --force!");
                return Err(eyre!("{} isn't built for this", profile.name));
            }
            profile.northstar = NorthstarSource::Overlayed;

            // remove all sources related to northstar installs
            profile.sources.retain(|source| match source {
                Source::DiscordRPC(_) | Source::Launcher(_) | Source::CoreMods(_) => false,
                Source::Mod(_) | Source::ModRepo(_) | Source::Package(_) => true,
            });

            profile.sources.extend_from_slice(&[
                Source::Launcher(LauncherSource::FromCommit(
                    fetch_latest(launcher.unwrap_or_else(|| {
                        "https://github.com/R2Northstar/NorthstarLauncher"
                            .try_into()
                            .unwrap()
                    }))
                    .await
                    .wrap_err("couldn't get latest launcher")?,
                )),
                Source::CoreMods(CoreModsSource::FromCommit(
                    fetch_latest(mods.unwrap_or_else(|| {
                        "https://github.com/R2Northstar/NorthstarMods"
                            .try_into()
                            .unwrap()
                    }))
                    .await
                    .wrap_err("couldn't get latest mods")?,
                )),
                Source::DiscordRPC(DiscordRPCSource::FromCommit(
                    fetch_latest(discord_rpc.unwrap_or_else(|| {
                        "https://github.com/R2Northstar/NorthstarDiscordRPC"
                            .try_into()
                            .unwrap()
                    }))
                    .await
                    .wrap_err("couldn't get latest discord rpc plugin")?,
                )),
            ]);

            bootstrap_northstar(profile, flightcore_ng_core::setup::northstar::Check::Force)
                .await
                .wrap_err("could not install northstar")?;
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
            launch_northstar(
                &settings,
                profile.as_deref().unwrap_or("R2NorthstarStable"),
                passthrough,
            )
            .await?;
        }

        Commands::Edit { editor } => {
            settings.save().await.wrap_err("tried saving settings")?;
            drop(settings);
            _ = tokio::process::Command::new(
                editor
                    .or_else(|| std::env::var("EDITOR").ok())
                    .unwrap_or_else(|| {
                        if cfg!(target_os = "linux") {
                            "nano".to_string()
                        } else {
                            "notepad".to_string()
                        }
                    }),
            )
            .arg(flightcore_ng_core::settings::SETTINGS_PATH.as_path())
            .output()
            .await?;
        }
        Commands::Open { profile: _ } => {
            open::that_detached(titanfall_path.join(profile))?;
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
    settings.add_default_profiles();

    titanfall
}
