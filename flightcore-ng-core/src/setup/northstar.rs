use color_eyre::eyre::{Context, Report, eyre};
use futures_lite::StreamExt;
use octocrab::models::repos::Asset;
use pelite::{FileMap, pe64::Pe as _};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, io::Cursor, path::Path};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{error, info};

use crate::{
    dev::{
        fetch_releases::{self, fetch_asset, fetch_latest_version},
        install_northstar::{NorthstarInstallInfo, get_northstar_from_revs},
    },
    settings::ProfileSettings,
    tmp_dir,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum Check {
    Force,
    Skip,
    Check,
}

const CORE_MODS: [&str; 3] = [
    "mods/Northstar.Client",
    "mods/Northstar.Custom",
    "mods/Northstar.CustomServers",
];

pub async fn bootstrap_northstar(profile: &ProfileSettings, check: Check) -> Result<(), Report> {
    if check == Check::Skip {
        info!("skipping bootstrap");
        return Ok(());
    }

    _ = fs::create_dir_all(tmp_dir()?).await;
    let _lock = fs::File::create(tmp_dir()?.join(".lock")).await?;

    match &profile.flavor {
        crate::settings::NorthstarSource::Version(version) => 'version: {
            // yes this is not very efficient but it's either here or in stable
            if check_if_installed(profile, version.to_string().as_str()).await
                && check == Check::Force
            {
                info!("bootstrap: version already matches doing nothing");
                break 'version;
            }

            download_northstar_version(profile, version).await?;
        }
        crate::settings::NorthstarSource::Stable => 'stable: {
            let version = match fetch_latest_version("R2Northstar", "Northstar")
                .await
                .wrap_err("couldn't fetch latest northstar version")
            {
                Err(err) => {
                    // could be because of network issue so an update isn't even possible anyway
                    error!("{err}");
                    return Ok(());
                }
                Ok(version) => version,
            };

            if check_if_installed(profile, &version).await {
                info!("bootstrap: version already matches doing nothing");
                break 'stable;
            }

            download_northstar_latest(profile).await?;
        }
        crate::settings::NorthstarSource::Nightly => 'nightly: {
            'version_check: {
                // break if the profile doesn't even exist
                if !profile.titanfall2_path.join(&profile.name).exists() {
                    break 'version_check;
                }

                let version_file_path = profile
                    .titanfall2_path
                    .join(&profile.name)
                    .join("nightly-ver");
                let mut file = match fs::File::open(&version_file_path).await {
                    Ok(file) => file,
                    Err(err) => match fs::File::create_new(version_file_path)
                        .await
                        .wrap_err(err)
                        .wrap_err("couldn't open version file for nightly build")
                    {
                        Ok(file) => file,
                        Err(err) => {
                            error!("{err}");
                            error!("no updates can be done!");
                            break 'nightly;
                        }
                    },
                };
                let mut version = String::new();
                _ = file.read_to_string(&mut version).await;

                let latest_version = fetch_releases::fetch_latest("catornot", "northstar-nightly")
                    .await?
                    .into_iter()
                    .find(|asset| asset.name.contains("northstar-nightly"))
                    .map(|asset| asset.name)
                    .unwrap_or_default();
                if version == latest_version {
                    info!("bootstrap: nightly version seems to be matching doing nothing");
                    break 'nightly;
                }

                _ = file.write_all_buf(&mut latest_version.as_bytes()).await;
            }

            download_latest_nightly(profile).await?;
        }
        crate::settings::NorthstarSource::Overlayed => {
            if check == Check::Check {
                info!(
                    "northstar with overlay will not self bootstrap since it's not possible to tell trivially if things have changed or not"
                );
            } else {
                let launcher = profile
                    .sources
                    .iter()
                    .find_map(|source| source.as_launcher());
                let mods = profile
                    .sources
                    .iter()
                    .find_map(|source| source.as_core_mods());
                let discord_rpc = profile
                    .sources
                    .iter()
                    .find_map(|source| source.as_discord_rpc());

                let (Some(launcher), Some(mods)) = (launcher, mods) else {
                    return Err(eyre!(
                        "cannot bootstrap northstar in overlay mode without launcher and mods specified"
                    ));
                };

                match (launcher, mods, discord_rpc) {
                    (
                        crate::settings::LauncherSource::FromCommit(launcher),
                        crate::settings::CoreModsSource::FromCommit(mods),
                        Some(crate::settings::DiscordRPCSource::FromCommit(discord_rpc)),
                    ) => {
                        install_northstar(
                            &get_northstar_from_revs(
                                NorthstarInstallInfo::new(mods.clone(), launcher.clone())
                                    .with_discord_rpc(discord_rpc.clone()),
                            )
                            .await?,
                            &profile.name,
                            &profile.titanfall2_path,
                        )
                        .await?;
                    }
                    (
                        crate::settings::LauncherSource::FromCommit(launcher),
                        crate::settings::CoreModsSource::FromCommit(mods),
                        None,
                    ) => {
                        install_northstar(
                            &get_northstar_from_revs(NorthstarInstallInfo::new(
                                mods.clone(),
                                launcher.clone(),
                            ))
                            .await?,
                            &profile.name,
                            &profile.titanfall2_path,
                        )
                        .await?;
                    }
                    (
                        crate::settings::LauncherSource::Path(launcher),
                        crate::settings::CoreModsSource::Path(mods),
                        _,
                    ) => {
                        #[cfg(target_os = "linux")]
                        {
                            _ = fs::remove_file(
                                profile
                                    .titanfall2_path
                                    .join(&profile.name)
                                    .join("Northstar.dll"),
                            )
                            .await;
                            _ = fs::symlink(
                                launcher.join("build").join("game").join("Northstar.dll"),
                                profile
                                    .titanfall2_path
                                    .join(&profile.name)
                                    .join("Northstar.dll"),
                            )
                            .await;
                            for (top, mod_path) in CORE_MODS
                                .into_iter()
                                .map(|path| profile.titanfall2_path.join(&profile.name).join(path))
                                .filter_map(|path| {
                                    Some((
                                        path.components().next()?.as_os_str().to_os_string(),
                                        path,
                                    ))
                                })
                            {
                                _ = fs::remove_file(&mod_path).await;
                                _ = fs::symlink(mods.join(top), mod_path).await;
                            }
                        }
                    }
                    _ => todo!("other overlays are not supported yet"),
                };
            }
        }
        crate::settings::NorthstarSource::Ion => 'ion: {
            let version = match fetch_latest_version("r2ion", "Ion")
                .await
                .wrap_err("couldn't fetch latest ion version")
            {
                Err(err) => {
                    // could be because of network issue so an update isn't even possible anyway
                    error!("{err}");
                    return Ok(());
                }
                Ok(version) => version,
            };

            if check_if_installed(profile, &version).await {
                info!("bootstrap: version already matches doing nothing");
                break 'ion;
            }

            download_ion_latest(profile).await?;
        }
    }

    Ok(())
}

async fn download_northstar_latest(profile: &ProfileSettings) -> Result<(), Report> {
    let northstar_asset = fetch_releases::fetch_latest("R2Northstar", "Northstar")
        .await?
        .into_iter()
        .find(|asset| asset.name.contains("Northstar.release"))
        .ok_or_else(|| eyre!("no northstar found for latest release somehow"))?;

    info!("installing northstar version {}", northstar_asset.name);

    install_northstar_release_asset(profile, northstar_asset).await?;

    Ok(())
}

async fn download_northstar_version(
    profile: &ProfileSettings,
    version: &Version,
) -> Result<(), Report> {
    let northstar_asset =
        fetch_releases::fetch_version("R2Northstar", "Northstar", &version.to_string())
            .await?
            .into_iter()
            .find(|asset| asset.name.contains("Northstar.release"))
            .ok_or_else(|| eyre!("no northstar found for {version}"))?;

    info!("installing northstar version {}", northstar_asset.name);

    install_northstar_release_asset(profile, northstar_asset).await?;

    Ok(())
}

async fn download_latest_nightly(profile: &ProfileSettings) -> Result<(), Report> {
    let northstar_asset = fetch_releases::fetch_latest("catornot", "northstar-nightly")
        .await?
        .into_iter()
        .find(|asset| asset.name.contains("northstar-nightly"))
        .ok_or_else(|| eyre!("no northstar found for latest release somehow"))?;

    info!("installing nightly : {}", northstar_asset.name);

    install_northstar_release_asset(profile, northstar_asset).await?;

    Ok(())
}

async fn download_ion_latest(profile: &ProfileSettings) -> Result<(), Report> {
    let ion_asset = fetch_releases::fetch_latest("r2ion", "Ion")
        .await?
        .into_iter()
        .find(|asset| asset.name.contains("Ion.release"))
        .ok_or_else(|| eyre!("no northstar found for latest release somehow"))?;

    info!("installing ion version {}", ion_asset.name);

    install_northstar_release_asset(profile, ion_asset).await?;

    Ok(())
}

async fn install_northstar_release_asset(
    profile: &ProfileSettings,
    northstar_asset: Asset,
) -> Result<(), Report> {
    let bytes = fetch_asset(northstar_asset)
        .await
        .wrap_err("no assets found")?;
    let cursor = Cursor::new(bytes);
    let tmp_dir = tmp_dir()?.join("northstar-release");
    _ = fs::remove_dir_all(&tmp_dir).await;

    zip::ZipArchive::new(cursor)
        .wrap_err("couldn't get a zip archive from the northstar download")?
        .extract(&tmp_dir)
        .wrap_err("couldn't extract the northstar download from the zip archive")?;
    install_northstar(&tmp_dir, &profile.name, &profile.titanfall2_path).await?;
    Ok(())
}

pub async fn install_northstar(
    northstar_dir: &Path,
    profile: &str,
    titanfall_dir: &Path,
) -> Result<(), Report> {
    let r2northstar_tmp = northstar_dir.join("R2Northstar");
    let r2northstar_dst = titanfall_dir.join(profile);
    let bin_tmp = northstar_dir.join("bin");
    let bin_dst = titanfall_dir.join("bin");

    // create the profile
    _ = fs::create_dir(&r2northstar_dst).await;

    // since files can be removed we must delete these mods
    for path in CORE_MODS
        .iter()
        .map(|path| r2northstar_dst.join(path))
        .filter(|path| path.exists())
    {
        fs::remove_dir_all(path).await?;
    }

    // copy r2northstar
    let mut entries = async_walkdir::WalkDir::new(&r2northstar_tmp);
    while let Some(path) = entries.next().await {
        let path = path.wrap_err("couldn't get path")?.path();

        let copy_path = r2northstar_dst.join(
            path.strip_prefix(&r2northstar_tmp)
                .wrap_err("couldn't get relative path")?,
        );

        if path.is_dir() {
            fs::create_dir_all(&copy_path)
                .await
                .wrap_err("failed to create a new directory in install path")?;
        } else if path.is_file() {
            if copy_path.exists() {
                fs::remove_file(&copy_path)
                    .await
                    .wrap_err_with(|| eyre!("failed to delete file {copy_path:?}"))?;
            }
            fs::copy(&path, &copy_path)
                .await
                .wrap_err_with(|| eyre!("failed to copy file from tmp directory to install directory : {path:?} to {copy_path:?}"))?;

            // when installing from nix store the permissions are all read only
            #[cfg(target_os = "linux")]
            if let Ok(file) = fs::File::open(copy_path).await {
                use std::os::unix::fs::PermissionsExt;

                if let Ok(mut permissions) =
                    file.metadata().await.map(|metadata| metadata.permissions())
                {
                    permissions.set_mode(0o6444);
                }
            }
        }
    }

    // copy northstar.dll
    if northstar_dir.join("Northstar.dll").exists() {
        let northstar_dst = r2northstar_dst.join("Northstar.dll");
        let pdb_dst = r2northstar_dst.join("Northstar.pdb");
        if northstar_dst.exists() {
            fs::remove_file(&northstar_dst)
                .await
                .wrap_err_with(|| eyre!("failed to delete file {northstar_dst:?}"))?;
        }
        if pdb_dst.exists() {
            fs::remove_file(&pdb_dst)
                .await
                .wrap_err_with(|| eyre!("failed to delete file {pdb_dst :?}"))?;
        }
        fs::copy(northstar_dir.join("Northstar.dll"), northstar_dst).await?;
        _ = fs::copy(northstar_dir.join("Northstar.pdb"), pdb_dst).await;
    }

    // copy eossdk.dll (for ion)
    if northstar_dir.join("EOSSDK-Win64-Shipping.dll").exists() {
        let eos_dst = r2northstar_dst.join("EOSSDK-Win64-Shipping.dll");
        if eos_dst.exists() {
            fs::remove_file(&eos_dst)
                .await
                .wrap_err_with(|| eyre!("failed to delete file {eos_dst:?}"))?;
        }
        fs::copy(northstar_dir.join("EOSSDK-Win64-Shipping.dll"), eos_dst).await?;
    }

    // copy launcher
    if northstar_dir.join("NorthstarLauncher.exe").exists() {
        let northstar_dst = titanfall_dir.join("NorthstarLauncher.exe");
        let pdb_dst = titanfall_dir.join("NorthstarLauncher.pdb");
        if northstar_dst.exists() {
            fs::remove_file(&northstar_dst)
                .await
                .wrap_err_with(|| eyre!("failed to delete file {northstar_dst:?}"))?;
        }
        if pdb_dst.exists() {
            fs::remove_file(&pdb_dst)
                .await
                .wrap_err_with(|| eyre!("failed to delete file {pdb_dst :?}"))?;
        }
        fs::copy(
            northstar_dir.join("NorthstarLauncher.exe"),
            titanfall_dir.join("NorthstarLauncher.exe"),
        )
        .await?;
        _ = fs::copy(
            northstar_dir.join("NorthstarLauncher.pdb"),
            titanfall_dir.join("NorthstarLauncher.pdb"),
        )
        .await;
    }

    // copy bin
    let mut entries = async_walkdir::WalkDir::new(&bin_tmp);
    while let Some(path) = entries.next().await {
        let path = path.wrap_err("couldn't get path")?.path();

        let copy_path = bin_dst.join(
            path.strip_prefix(&bin_tmp)
                .wrap_err("couldn't get relative path")?,
        );

        if path.is_dir() {
            fs::create_dir_all(&copy_path)
                .await
                .wrap_err("failed to create a new directory in install path")?;
        } else if path.is_file() {
            if copy_path.exists() {
                fs::remove_file(&copy_path)
                    .await
                    .wrap_err_with(|| eyre!("failed to delete file {copy_path:?}"))?;
            }
            fs::copy(&path, &copy_path)
                .await
                .wrap_err_with(|| eyre!("failed to copy file from tmp directory to install directory : {path:?} to {copy_path:?}"))?;
        }
    }

    Ok(())
}

async fn check_if_installed(profile: &ProfileSettings, version: &str) -> bool {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
    #[serde(rename_all = "PascalCase")]
    pub struct ModStub {
        pub version: String,
        #[serde(flatten)]
        pub _extra: HashMap<String, Value>,
    }

    let profile_path = profile.titanfall2_path.join(&profile.name);
    if !profile_path.exists() {
        return false;
    }

    // check version of core mods
    if CORE_MODS
        .iter()
        .any(|mod_path| !profile_path.join(mod_path).exists())
        && CORE_MODS.iter().any(|mod_path| {
            // non async but hopefully it's fine
            !std::fs::read_to_string(profile_path.join(mod_path).join("mod.json"))
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .map(|stub: ModStub| stub.version == version)
                .unwrap_or_default()
        })
    {
        return false;
    }

    // check version of launcher
    let Ok(dll) = FileMap::open(&profile_path.join("Northstar.dll")) else {
        return false;
    };

    Some(pelite::pe64::PeFile::from_bytes(&dll).unwrap())
        .and_then(|pe| pe.resources().ok())
        .and_then(|resources| resources.version_info().ok())
        .and_then(|version_info| {
            let lang = version_info.translation().first()?;
            let mut correct_version = false;
            version_info.strings(*lang, |key, value| {
                if key == "FileVersion" && value.strip_prefix("v").unwrap_or(value) == version {
                    correct_version = true;
                }
            });
            Some(correct_version)
        })
        .unwrap_or(true)
}
