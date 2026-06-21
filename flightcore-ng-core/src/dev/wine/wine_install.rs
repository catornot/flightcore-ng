use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Report, eyre};
use flate2::bufread::GzDecoder;
use futures_lite::StreamExt;
use tar::Archive;
use tokio::fs;
use tracing::info;

use crate::{
    dev::{
        fetch_releases,
        wine::{proton_dir, run_wine_command, wine_dir},
    },
    local_dir,
};

pub fn is_wine_installed() -> bool {
    proton_dir()
        .as_ref()
        .map(PathBuf::as_path)
        .map(Path::exists)
        .unwrap_or_default()
        && wine_dir()
            .as_ref()
            .map(PathBuf::as_path)
            .map(Path::exists)
            .unwrap_or_default()
}

pub async fn install_wine(
    mut install_progress_reporter: Option<impl FnMut(f32)>,
) -> Result<(), Report> {
    // install proton if it's not installed already
    if !proton_dir()
        .as_ref()
        .map(PathBuf::as_path)
        .map(Path::exists)
        .unwrap_or_default()
    {
        install_proton(install_progress_reporter).await?;
    } else {
        info!(
            "proton is already installed at {}",
            proton_dir().unwrap_or_default().display()
        );
        if let Some(install_progress_reporter) = install_progress_reporter.as_mut() {
            install_progress_reporter(1.0)
        }
    }

    info!("setting up wine prefix {}", wine_dir()?.display());
    // setup wine prefix
    fs::create_dir_all(wine_dir()?).await?;
    run_wine_command("", [""].into_iter(), None, None).await?;

    Ok(())
}

async fn install_proton(
    mut install_progress_reporter: Option<impl FnMut(f32)>,
) -> Result<(), Report> {
    if let Some(proton) = env::var("PROTON_PATH").ok().map(PathBuf::from) {
        fs::create_dir_all(proton_dir()?)
            .await
            .wrap_err("couldn't create proton dir")?;

        let total_paths = async_walkdir::WalkDir::new(&proton).count().await;
        let mut entries = async_walkdir::WalkDir::new(&proton);
        let mut enumeration = 0usize..;

        while let (Some(path), Some(current_index)) = (entries.next().await, enumeration.next()) {
            let path = path.wrap_err("couldn't get path in proton")?.path();

            let copy_path = proton_dir()?.join(
                path.strip_prefix(&proton)
                    .wrap_err("couldn't get relative path for proton")?,
            );

            if path.is_dir() {
                fs::create_dir_all(&copy_path)
                    .await
                    .wrap_err("failed to create a new directory in proton install path")?;
            } else if path.is_file() {
                if copy_path.exists() {
                    fs::remove_file(&copy_path).await.wrap_err_with(|| {
                        eyre!("failed to delete file in previous proton install {copy_path:?}")
                    })?;
                }
                fs::copy(&path, &copy_path)
            .await
            .wrap_err_with(|| eyre!("failed to copy file from proton directory to proton install directory : {path:?} to {copy_path:?}"))?;
            }

            if let Some(install_progress_reporter) = install_progress_reporter.as_mut() {
                install_progress_reporter(total_paths as f32 / current_index as f32)
            }
        }

        if let Some(install_progress_reporter) = install_progress_reporter.as_mut() {
            install_progress_reporter(1.0)
        }
    } else {
        let proton = fetch_releases::fetch_latest("GloriousEggroll", "proton-ge-custom")
            .await?
            .into_iter()
            .find(|proton| proton.name.starts_with("GE-Proton") && proton.name.ends_with(".tar.gz"))
            .ok_or_else(|| eyre!("couldn't find proton in the latest release"))?;

        info!("downloading proton {}", proton.name);
        let proton_extract_path = local_dir()?
            .join(
                proton
                    .name
                    .split_once('.')
                    .map(|(left, _)| left)
                    .unwrap_or(&proton.name),
            )
            .with_extension("");

        info!("extract path {}", proton_extract_path.display());

        let bytes = fetch_releases::fetch_asset(proton, install_progress_reporter).await?;
        let mut archive = Archive::new(GzDecoder::new(&*bytes));
        archive.set_overwrite(true);

        archive.unpack(local_dir()?)?;

        info!(
            "move {} to {}",
            proton_extract_path.display(),
            proton_dir()?.display()
        );
        // the archive unpacks it into a dir so it has to be moved into the right place
        fs::rename(proton_extract_path, proton_dir()?).await?;
    }

    Ok(())
}

pub async fn remove_wine() -> Result<(), Report> {
    _ = fs::remove_dir_all(proton_dir()?).await;
    _ = fs::remove_dir_all(wine_dir()?).await;
    Ok(())
}
