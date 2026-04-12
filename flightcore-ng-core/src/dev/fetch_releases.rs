use bytes::Bytes;
use color_eyre::eyre::Report;
use octocrab::models::repos::Asset;

pub async fn fetch_latest(owner: &str, repo: &str) -> Result<Vec<Asset>, Report> {
    let instance = octocrab::instance();
    let repo = instance.repos(owner, repo);
    let latest = repo.releases().get_latest().await?;

    Ok(latest.assets)
}

pub async fn fetch_asset(asset: Asset) -> Result<Bytes, Report> {
    Ok(reqwest::get(asset.browser_download_url.to_string())
        .await?
        .bytes()
        .await?)
}
