use color_eyre::eyre::{Result, WrapErr, eyre};
use octocrab::models::repos::RepoCommit;
use reqwest::Url;

pub async fn fetch_latest(url: Url) -> Result<RepoCommit> {
    match fetch_latest_from_pr(url.clone()).await {
        Err(err) => fetch_latest_from_repo(url).await.with_context(|| err),
        ok => ok,
    }
}

async fn fetch_latest_from_pr(pr: Url) -> Result<RepoCommit> {
    let pr_split = pr
        .path()
        .split('/')
        .take(5)
        .filter(|str| !str.is_empty())
        .collect::<Vec<&str>>();
    let ["R2Northstar", repo, "pull", number] = pr_split
        .as_array()
        .copied()
        .ok_or_else(|| eyre!("invalid pr url : {}", pr.to_string()))?
    else {
        return Err(eyre!(
            "not pull request url or owner isn't R2Northstar : {}",
            pr.to_string()
        ));
    };

    let number = number.parse::<u64>().wrap_err("not a valid pr number")?;

    octocrab::instance()
        .pulls("R2Northstar", repo)
        .pr_commits(number)
        .per_page(1)
        .page(1u32)
        .send()
        .await
        .wrap_err("fetch pr failed")?
        .items
        .first()
        .cloned()
        .ok_or_else(|| eyre!("no commits"))
}

async fn fetch_latest_from_repo(url: Url) -> Result<RepoCommit> {
    let items = url
        .path()
        .split('/')
        .take(4)
        .filter(|str| !str.is_empty())
        .collect::<Vec<&str>>();
    let repo_split = items;
    let ["R2Northstar", repo] = repo_split
        .as_array()
        .copied()
        .ok_or_else(|| eyre!("invalid repo url : {}", url.to_string()))?
    else {
        return Err(eyre!("owner isn't R2Northstar : {}", url.to_string()));
    };

    octocrab::instance()
        .repos("R2Northstar", repo)
        .list_commits()
        .per_page(1)
        .page(1u32)
        .send()
        .await
        .wrap_err("fetch pr failed")?
        .items
        .first()
        .cloned()
        .ok_or_else(|| eyre!("no commits"))
}
