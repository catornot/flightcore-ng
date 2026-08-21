use std::pin::Pin;

use color_eyre::eyre::{Report, eyre};

use crate::{
    TITANFALL_ID,
    dev::wine::wine_run,
    settings::{FlightCoreSettings, LaunchMethod},
    setup::setup_profile,
};

pub async fn launch_northstar(
    settings: &FlightCoreSettings,
    profile: &str,
    mut launch_args: Vec<String>,
) -> Result<Pin<Box<dyn Future<Output = Result<(), Report>> + 'static + Send>>, Report> {
    let profile = settings
        .get_profile(profile)
        .ok_or_else(|| eyre!("profile({profile}) doesn't exist"))?;

    setup_profile(profile).await?;

    let launch = if profile.launch_method == LaunchMethod::Any {
        if cfg!(target_os = "linux") && settings.settings.preferred_launch == LaunchMethod::Steam {
            LaunchMethod::Steam
        } else if cfg!(target_os = "linux") {
            LaunchMethod::Wine
        } else {
            LaunchMethod::Direct
        }
    } else {
        profile.launch_method
    };

    launch_args.extend(profile.launch_args.clone());
    if !profile.ignore_global_launch_args {
        launch_args.extend(settings.settings.launch_args.clone());
    }

    let exe = profile.titanfall2_path.join("NorthstarLauncher.exe");
    let runner: Pin<Box<dyn Future<Output = Result<(), Report>> + Send>> = match launch {
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
            Box::pin(wait_for_northstar())
        }
        LaunchMethod::Wine => {
            Box::pin(async move { wine_run::run_game(&exe, &launch_args, false, false).await })
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
            Box::pin(wait_for_northstar())
        }
        LaunchMethod::Any | LaunchMethod::Direct => {
            Box::pin(async move { wine_run::run_game(&exe, &launch_args, false, false).await })
        }
    };

    Ok(runner)
}

async fn wait_for_northstar() -> Result<(), Report> {
    Ok(())
}
