use bytes::Bytes;
use color_eyre::eyre::Report;
use eyre::eyre;
use octocrab::models::repos::Asset;

pub async fn fetch_latest(owner: &str, repo: &str) -> Result<Vec<Asset>, Report> {
    let instance = octocrab::instance();
    let repo = instance.repos(owner, repo);
    let latest = repo.releases().get_latest().await?;

    Ok(latest.assets)
}

pub async fn fetch_version(owner: &str, repo: &str, version: &str) -> Result<Vec<Asset>, Report> {
    let instance = octocrab::instance();
    let repo = instance.repos(owner, repo);
    // TODO: figure out if this is the correct approach since we
    let page = repo.releases().list().send().await?;
    let release = page
        .items
        .iter()
        .find(|release| {
            release
                .tag_name
                .strip_prefix("v")
                .unwrap_or(&release.tag_name)
                == version
        })
        .ok_or_else(|| eyre!("couldn't find the release for version : {version}"))?;

    Ok(release.assets.clone())
}

pub async fn fetch_asset(asset: Asset) -> Result<Bytes, Report> {
    Ok(reqwest::get(asset.browser_download_url.to_string())
        .await?
        .bytes()
        .await?)
}

pub async fn fetch_latest_version(owner: &str, repo: &str) -> Result<String, Report> {
    let instance = octocrab::instance();
    let repo = instance.repos(owner, repo);
    // TODO: figure out if this is the correct approach since we
    let latest = repo.releases().get_latest().await?;

    Ok(latest
        .tag_name
        .strip_prefix("v")
        .map(ToString::to_string)
        .unwrap_or(latest.tag_name))
}
