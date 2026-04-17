use color_eyre::eyre::{Report, eyre};
use eyre::Context;
use nix_compat::flakeref::FlakeRef;
use reqwest::Url;
use std::{path::PathBuf, process::Stdio};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use tracing::info;

use crate::{dev::fetch_revs::fetch_latest, tmp_dir};

pub struct NorthstarInstallInfo {
    mods_sha: String,
    launcher_sha: String,
    rpc_sha: Option<String>,
}

impl NorthstarInstallInfo {
    pub fn new(mods: String, launcher: String) -> Self {
        Self {
            mods_sha: mods,
            launcher_sha: launcher,
            rpc_sha: None,
        }
    }

    pub async fn try_from_url(mods: Url, launcher: Url) -> Result<Self, Report> {
        Ok(Self {
            mods_sha: fetch_latest(mods).await?,
            launcher_sha: fetch_latest(launcher).await?,
            rpc_sha: None,
        })
    }

    pub fn with_discord_rpc(self, rpc: String) -> Self {
        Self {
            rpc_sha: Some(rpc),
            ..self
        }
    }
}

pub async fn get_northstar_from_revs(install: NorthstarInstallInfo) -> Result<PathBuf, Report> {
    if Command::new("nix").arg("--version").output().await.is_err() {
        return Err(eyre!(
            "nix package manager not installed : https://nixos.org/download"
        ));
    }
    if Command::new("git").arg("--version").output().await.is_ok() {
    } else {
        return Err(eyre!("git not installed"));
    }

    let launcher = get_repo_uri(
        install.launcher_sha,
        "https://github.com/R2Northstar/NorthstarLauncher.git",
    )?;
    let mods = get_github_uri(install.mods_sha, "R2Northstar", "NorthstarMods");
    let rpc = install
        .rpc_sha
        .map(|sha| get_github_uri(sha, "R2Northstar", "NorthstarMods"));

    let urls = [("mods", mods.as_str()), ("launcher", launcher.as_str())];
    let inputs = urls
        .into_iter()
        .chain(rpc.iter().map(|url| ("discordrpc", url.as_str())))
        .flat_map(|(input, uri)| ["--override-input", input, uri])
        .collect::<Vec<_>>();

    const EXPERIEMENTAL_FLAGS: [&str; 4] = [
        "--extra-experimental-features",
        "nix-command",
        "--extra-experimental-features",
        "flakes",
    ];

    fs::create_dir_all(tmp_dir()?).await?;
    let out_link = tmp_dir()?.join("northstar-dev");
    let flake_dir = tmp_dir()?.join("northstar-nightly");
    let lock = tmp_dir()?.join(".lock");

    let _lock = fs::File::create(lock).await?;
    _ = fs::remove_file(out_link.as_path()).await;

    let err = if flake_dir.exists() {
        info!("updating sources");
        let child = Command::new("git")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(&flake_dir)
            .args(["reset", "origin", "--hard"])
            .spawn()?;
        print_and_collect_errors(child, "git reset origin --hard")
            .await
            .err()
            .unwrap_or_else(|| eyre!(""))
    } else {
        info!("cloning");
        let child = Command::new("nix")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(tmp_dir()?)
            .arg("flake")
            .arg("clone")
            .arg("github:catornot/northstar-nightly")
            .args(EXPERIEMENTAL_FLAGS)
            .arg("--dest")
            .arg(&flake_dir)
            .spawn()?;
        print_and_collect_errors(child, "nix flake clone")
            .await
            .err()
            .unwrap_or_else(|| eyre!(""))
    };

    let child = Command::new("nix")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .current_dir(&flake_dir)
        .arg("flake")
        .arg("lock")
        .arg(".")
        .args(EXPERIEMENTAL_FLAGS)
        .args(inputs)
        .spawn()?;
    let err = print_and_collect_errors(child, "nix lock")
        .await
        .wrap_err(err)
        .err()
        .unwrap_or_else(|| eyre!(""));

    info!("building northstar with nix");
    let child = Command::new("nix")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .current_dir(&flake_dir)
        .arg("build")
        .args(EXPERIEMENTAL_FLAGS)
        .arg("--out-link")
        .arg(&out_link)
        .spawn()?;
    let err = print_and_collect_errors(child, "nix build")
        .await
        .wrap_err(err)
        .err()
        .unwrap_or_else(|| eyre!(""));

    info!(
        "built northstar :: mods : {}, launcher : {}",
        urls[0].1, urls[1].1
    );

    out_link.read_link().wrap_err(err)
}

fn get_repo_uri(sha: String, repo: &'static str) -> Result<Url, Report> {
    Ok(FlakeRef::Git {
        all_refs: false,
        export_ignore: false,
        keytype: None,
        public_key: None,
        public_keys: None,
        r#ref: None,
        rev: Some(sha),
        shallow: false,
        submodules: true,
        url: repo.try_into()?,
        verify_commit: false,
    }
    .to_uri())
}

fn get_github_uri(sha: String, owner: &'static str, repo: &'static str) -> String {
    format!("github:{owner}/{repo}/{sha}")
}

/// nix will put everything into stderr
async fn print_and_collect_errors(mut child: Child, command: &str) -> Result<(), Report> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| eyre!("couldn't capture stdout for {command}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| eyre!("couldn't capture stdout for {command}"))?;

    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.ok().flatten() {
            info!("{}", line);
        }
    });

    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();

        while let Some(line) = reader.next_line().await.ok().flatten() {
            info!("{}", line);
        }
    });

    child.wait().await?;
    stdout_handle.await?;
    stderr_handle.await?;

    Ok(())
}
