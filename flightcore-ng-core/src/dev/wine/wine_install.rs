use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Report, eyre};
use flate2::bufread::GzDecoder;
use tar::Archive;
use tokio::{fs, process::Command};
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

pub async fn install_wine() -> Result<(), Report> {
    // install proton
    if let Some(proton) = env::var("PROTON_PATH").ok().map(PathBuf::from) {
        // lmao
        // should maybe add one that isn't dependent on uutils
        Command::new("cp")
            .args(["-r".as_ref(), proton.as_os_str(), proton_dir()?.as_os_str()])
            .output()
            .await
            .wrap_err("couldn't copy proton install")?;
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

        let bytes = fetch_releases::fetch_asset(proton).await?;
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

    info!("setting up wine prefix {}", wine_dir()?.display());
    // setup wine prefix
    fs::create_dir_all(wine_dir()?).await?;
    run_wine_command("", [""].into_iter(), None, None).await?;

    Ok(())
}

pub async fn remove_wine() -> Result<(), Report> {
    _ = fs::remove_dir_all(proton_dir()?).await;
    _ = fs::remove_dir_all(wine_dir()?).await;
    Ok(())
}
