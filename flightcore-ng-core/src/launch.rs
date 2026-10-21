use color_eyre::eyre::{Report, eyre};
use std::{ffi::OsString, pin::Pin, str::FromStr, time::Duration};
use sysinfo::System;
use tokio::time::sleep;
use tracing::info;

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
) -> Result<Pin<Box<dyn Future<Output = Result<(), Report>> + 'static + Send>>, Report> {
    let profile = store
        .get_profile(profile)
        .ok_or_else(|| eyre!("profile({profile}) doesn't exist"))?;

    setup_profile(profile).await?;

    info!("using profile {}", profile.name);
    launch_args.push(format!("-profile={}", profile.name));

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
    let launch = get_launch_method(store, profile);

    launch_args.extend(profile.launch_args.clone());
    if !profile.ignore_global_launch_args {
        launch_args.extend_from_slice(&store.config.launch_args);
    }

    let exe = profile.titanfall2_path.join("NorthstarLauncher.exe");
    let runner: Pin<Box<dyn Future<Output = Result<(), Report>> + Send>> = match launch {
        LaunchMethod::Steam => {
            open::that_detached(format!(
                "steam://run/{}/ --northstar {}/",
                TITANFALL_ID,
                launch_args
                    .into_iter()
                    .map(|arg| arg + "  ")
                    .collect::<String>()
            ))?;
            Box::pin(wait_for_northstar_to_exit())
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
            Box::pin(wait_for_northstar_to_exit())
        }
        LaunchMethod::Any | LaunchMethod::Direct => {
            Box::pin(async move { wine_run::run_game(&exe, &launch_args, false, false).await })
        }
    };

    Ok(runner)
}

async fn wait_for_northstar_to_exit() -> Result<(), Report> {
    let system = System::new_all();

    let northstar_launcher =
        OsString::from_str("NorthstarLauncher.exe").expect("this should always work");
    let titanfall = OsString::from_str("Titanfall2.exe").expect("this should always work");
    while system
        .processes_by_name(&northstar_launcher)
        .next()
        .is_some()
        || system.processes_by_name(&titanfall).next().is_some()
    {
        sleep(Duration::from_secs(10)).await;
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
