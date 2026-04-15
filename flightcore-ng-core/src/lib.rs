use color_eyre::eyre::Report;
use eyre::{Context, eyre};
use std::path::{Path, PathBuf};
use tokio::fs;

pub mod dev;
pub mod settings;

pub const TITANFALL_ID: u32 = 1237970;

pub fn local_dir() -> Result<PathBuf, color_eyre::Report> {
    let dirs = directories::ProjectDirs::from("org", "flightcore", "flightcore-ng");
    let path = dirs.unwrap().data_local_dir().to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn tmp_dir() -> Result<PathBuf, color_eyre::Report> {
    Ok(local_dir()?.join("tmp"))
}

// TODO: move this
pub async fn install_northstar(
    northstar_dir: &Path,
    profile: &str,
    dst: &Path,
) -> Result<(), Report> {
    let r2northstar_tmp = northstar_dir.join("R2Northstar");
    let r2northstar_dst = dst.join(profile);
    let bin_tmp = northstar_dir.join("bin");
    let bin_dst = dst.join("bin");

    // since files can be removed we must delete these mods
    const DELETE_PATHS: [&str; 3] = [
        "mods/Northstar.Client",
        "mods/Northstar.Custom",
        "mods/Northstar.CustomServers",
    ];

    for path in DELETE_PATHS
        .iter()
        .map(|path| r2northstar_dst.join(path))
        .filter(|path| path.exists())
    {
        fs::remove_dir_all(path).await?;
    }

    use futures_lite::StreamExt;

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
        fs::copy(
            northstar_dir.join("Northstar.dll"),
            r2northstar_dst.join("Northstar.dll"),
        )
        .await?;
        _ = fs::copy(
            northstar_dir.join("Northstar.pdb"),
            r2northstar_dst.join("Northstar.pdb"),
        )
        .await;
    }

    // copy launcher
    if northstar_dir.join("NorthstarLauncher.exe").exists() {
        let northstar_dst = dst.join("NorthstarLauncher.exe");
        let pdb_dst = dst.join("NorthstarLauncher.pdb");
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
            dst.join("NorthstarLauncher.exe"),
        )
        .await?;
        _ = fs::copy(
            northstar_dir.join("NorthstarLauncher.pdb"),
            dst.join("NorthstarLauncher.pdb"),
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
