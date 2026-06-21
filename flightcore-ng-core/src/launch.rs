use color_eyre::eyre::{Report, eyre};

use crate::{
    TITANFALL_ID,
    dev::wine::wine_run,
    settings::LaunchMethod,
    setup::{setup_profile, sources::Store},
};

pub async fn launch_northstar(
    store: &Store,
    profile: &str,
    mut launch_args: Vec<String>,
) -> Result<(), Report> {
    let profile = store
        .get_profile(profile)
        .ok_or_else(|| eyre!("profile({profile}) doesn't exist"))?;

    setup_profile(profile).await?;

    let launch = get_launch_method(store, profile);

    launch_args.extend(profile.launch_args.clone());
    if !profile.ignore_global_launch_args {
        launch_args.extend_from_slice(&store.config.launch_args);
    }

    match launch {
        LaunchMethod::Steam => {
            open::that_detached(format!(
                "steam://run/{}//-profile={} --northstar {}/",
                TITANFALL_ID,
                profile.name,
                launch_args
                    .into_iter()
                    .map(|arg| arg + "  ")
                    .collect::<String>()
            ))?;
        }
        LaunchMethod::Wine => {
            wine_run::run_game(
                &profile.titanfall2_path.join("NorthstarLauncher.exe"),
                &launch_args,
                false,
            )
            .await?;
        }
        LaunchMethod::Direct | LaunchMethod::Any if cfg!(target_os = "windows") => {
            open::that_detached(format!(
                "{} -profile={} --northstar {}/",
                profile
                    .titanfall2_path
                    .join("NorthstarLauncher.exe")
                    .display(),
                profile.name,
                launch_args
                    .into_iter()
                    .map(|arg| arg + "  ")
                    .collect::<String>()
            ))?;
        }
        LaunchMethod::Any | LaunchMethod::Direct => {
            wine_run::run_game(
                &profile.titanfall2_path.join("NorthstarLauncher.exe"),
                &launch_args,
                false,
            )
            .await?;
        }
    }

    Ok(())
}

fn get_launch_method(store: &Store, profile: &crate::settings::ProfileConfig) -> LaunchMethod {
    if profile.launch_method == LaunchMethod::Any {
        if cfg!(target_os = "linux") && store.config.preferred_launch == LaunchMethod::Steam {
            LaunchMethod::Steam
        } else if cfg!(target_os = "linux") {
            LaunchMethod::Wine
        } else {
            LaunchMethod::Direct
        }
    } else {
        profile.launch_method
    }
}
