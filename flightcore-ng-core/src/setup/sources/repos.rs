use color_eyre::eyre::{Report, WrapErr, eyre};
use eyre::ContextCompat;
use git2::{Oid, Repository};
use nix_compat::flakeref::FlakeRef;
use snix_eval::Value;
use std::{
    borrow::Cow,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    setup::sources::{SourceImpl, Store},
    snix_ext::CloneableFlakeRef,
};

#[derive(Debug, Clone)]
pub struct Repo {
    url: CloneableFlakeRef,
    sub_mod: Option<Box<str>>,
    hash: Option<Box<str>>,
}

impl SourceImpl for Repo {
    fn get_name(&self) -> String {
        match &*self.url {
            FlakeRef::GitHub {
                owner,
                repo,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev,
            } => format!(
                "github-{owner}-{repo}-{}-{}{}",
                rev.as_ref().map(String::as_str).unwrap_or_default(),
                self.sub_mod.as_ref().map(|_| "-").unwrap_or_default(),
                self.sub_mod
                    .as_ref()
                    .map(|sub_mod| &sub_mod[..])
                    .unwrap_or_default(),
            ),
            FlakeRef::GitLab {
                owner,
                repo,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev,
            } => format!(
                "gitlab-{owner}-{repo}-{}-{}{}",
                rev.as_ref().map(String::as_str).unwrap_or_default(),
                self.sub_mod.as_ref().map(|_| "-").unwrap_or_default(),
                self.sub_mod
                    .as_ref()
                    .map(|sub_mod| &sub_mod[..])
                    .unwrap_or_default(),
            ),
            FlakeRef::Mercurial { r#ref, rev } => format!(
                "repo-{}-{}{}{}",
                r#ref.as_ref().expect("ref must be valid in repo source"),
                rev.as_ref().expect("rev must be valid in repo source"),
                self.sub_mod.as_ref().map(|_| "-").unwrap_or_default(),
                self.sub_mod
                    .as_ref()
                    .map(|sub_mod| &sub_mod[..])
                    .unwrap_or_default(),
            ),
            FlakeRef::Path {
                last_modified: _,
                nar_hash: _,
                path,
                rev: _,
                rev_count: _,
            } => path.to_string_lossy().replace(['\\', '/'], "-"),
            FlakeRef::SourceHut {
                owner,
                repo,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev,
            } => format!(
                "gitlab-{owner}-{repo}-{}-{}{}",
                rev.as_ref().map(String::as_str).unwrap_or_default(),
                self.sub_mod.as_ref().map(|_| "-").unwrap_or_default(),
                self.sub_mod
                    .as_ref()
                    .map(|sub_mod| &sub_mod[..])
                    .unwrap_or_default(),
            ),
            FlakeRef::Tarball {
                last_modified: _,
                nar_hash: _,
                rev: _,
                rev_count: _,
                url: _,
            } => todo!(),
            flake_ref => unreachable!("invalid flake ref in repo source {flake_ref:#?}"),
        }
    }

    fn install_path(&self, profile_path: &Path) -> Option<PathBuf> {
        Some(profile_path.join("mods"))
    }

    async fn instantiate(&self, store_path: &Path) -> Result<(), eyre::Error> {
        let name = self.get_name();
        let install_path = store_path.join(name);

        fs::remove_dir_all(&install_path);

        let repo = match self.url.deref() {
            FlakeRef::Git {
                all_refs: _,
                export_ignore: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: Some(rev),
                shallow: _,
                submodules,
                url,
                verify_commit: _,
            } => {
                let repo = (if *submodules {
                    Repository::clone_recurse
                } else {
                    Repository::clone
                })(url.as_str(), &install_path)?;

                Some((rev.parse::<Oid>()?, repo))
            }
            FlakeRef::GitHub {
                owner,
                repo,
                host,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: Some(rev),
            } => Some((
                rev.parse()?,
                Repository::clone(
                    &format!(
                        "https://{}/{owner}/{repo}.git",
                        host.as_ref().map(String::as_str).unwrap_or("github.com")
                    ),
                    &install_path,
                )?,
            )),
            FlakeRef::GitLab {
                owner,
                repo,
                host,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: Some(rev),
            } => Some((
                rev.parse()?,
                Repository::clone(
                    &format!(
                        "https://{}/{owner}/{repo}.git",
                        host.as_ref().map(String::as_str).unwrap_or("gitlab.com")
                    ),
                    &install_path,
                )?,
            )),
            FlakeRef::Mercurial {
                r#ref: Some(url),
                rev: Some(rev),
            } => Some((rev.parse()?, Repository::clone(url, &install_path)?)),
            FlakeRef::SourceHut {
                owner,
                repo,
                host,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: Some(rev),
            } => Some((
                rev.parse()?,
                Repository::clone(
                    &format!(
                        "https://{}/{owner}/{repo}.git",
                        host.as_ref().map(String::as_str).unwrap_or("sourcehut.org")
                    ),
                    &install_path,
                )?,
            )),
            FlakeRef::Git {
                all_refs: _,
                export_ignore: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: None,
                shallow: _,
                submodules: _,
                url: _,
                verify_commit: _,
            }
            | FlakeRef::GitHub {
                owner: _,
                repo: _,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: None,
            }
            | FlakeRef::GitLab {
                owner: _,
                repo: _,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: None,
            }
            | FlakeRef::Mercurial {
                r#ref: _,
                rev: None,
            }
            | FlakeRef::SourceHut {
                owner: _,
                repo: _,
                host: _,
                keytype: _,
                public_key: _,
                public_keys: _,
                r#ref: _,
                rev: None,
            } => Err(eyre!(
                "invalid flake ref missing rev for remote repo {:?}",
                self.url
            ))?,
            FlakeRef::Tarball {
                last_modified: _,
                nar_hash: _,
                rev: _,
                rev_count: _,
                url: _,
            } => todo!(),
            FlakeRef::Path {
                last_modified: _,
                nar_hash: _,
                path: _,
                rev: _,
                rev_count: _,
            } => None,
            flake_ref => Err(eyre!("invalid flake ref for remote repo {:?}", flake_ref))?,
        };
        if let Some((rev, repo)) = repo {
            let commit = repo.find_commit(rev)?;
            repo.checkout_tree(commit.as_object(), None)?;

            Ok(())
        } else if let FlakeRef::Path {
            last_modified: _,
            nar_hash: _,
            path,
            rev: _,
            rev_count: _,
        } = self.url.as_ref()
        {
            crate::symlink(
                self.sub_mod
                    .as_ref()
                    .map(|sub_mod| Cow::Owned(path.join(sub_mod.as_ref())))
                    .unwrap_or_else(|| Cow::Borrowed(path.as_path())),
                install_path,
            )
            .await
            .wrap_err_with(|| eyre!("couldn't create reference to local repo; possibly wrong path"))
        } else {
            Err(eyre!(
                "{} has bad state; can't instantiate",
                self.get_name()
            ))
        }
    }

    async fn is_instantiated(&self, store_path: &Path) -> bool {
        // TODO: check if repo is sound (aka on the right commit)
        self.sub_mod
            .as_ref()
            .map(|sub_mod| Cow::Owned(store_path.join(sub_mod.as_ref())))
            .unwrap_or_else(|| Cow::Borrowed(store_path))
            .exists()
    }
}

impl Repo {
    pub fn try_from_value<'a>(source: &Value) -> Result<Option<Self>, Report> {
        let source = match source.to_attrs() {
            Ok(source) => source,
            Err(err) => return Err(eyre!(err.to_string())),
        };
        if source
            .select("_type")
            .map(|ty| ty.to_string() == "repo")
            .unwrap_or_default()
        {
            return Ok(None);
        }

        let hash = invert_result_option(source.select("hash").map(|value| value.to_str()))
            .map_err(|err| eyre!(err.to_string()))
            .wrap_err_with(|| eyre!("failed to get hash as a string"))?
            .map(|nix_str| nix_str.to_string())
            .map(|nix_str| Box::from(nix_str.as_str()));

        let repo = Self {
            url: source
                .select("url")
                .map(|value| value.to_string().parse::<FlakeRef>())
                .wrap_err_with(|| eyre!("failed to find url for repo type"))?
                .wrap_err_with(|| eyre!("failed to parse the url as a flake ref"))?
                .into(),
            sub_mod: invert_result_option(source.select("sub-mod").map(|value| value.to_path()))
                .map_err(|err| eyre!(err.to_string()))
                .wrap_err_with(|| eyre!("wrong type for sub_mod"))?
                .map(|path| Box::from(path.to_string_lossy())),
            hash,
        };

        Ok(Some(repo))
    }
}

fn invert_result_option<R, E>(optional: Option<Result<R, E>>) -> Result<Option<R>, E> {
    match optional {
        Some(real) => real.map(Some),
        None => Ok(None),
    }
}

/*
for progress bar for repos use `Unpacking objects: <percentage>%`

remote: Enumerating objects: 18, done.
remote: Counting objects: 100% (18/18), done.
remote: Compressing objects: 100% (17/17), done.
remote: Total 18 (delta 10), reused 2 (delta 1), pack-reused 0 (from 0)
Unpacking objects: 100% (18/18), 13.60 KiB | 3.40 MiB/s, done.
From github.com:R2Northstar/NorthstarDiscordRPC
   41cf32d..3d836fe  main                                -> origin/main
 * [new branch]      dependabot/cargo/discord-sdk-0.4.0  -> origin/dependabot/cargo/discord-sdk-0.4.0
 * [new branch]      dependabot/cargo/parking_lot-0.12.5 -> origin/dependabot/cargo/parking_lot-0.12.5
 * [new branch]      dependabot/cargo/tokio-1.52.3       -> origin/dependabot/cargo/tokio-1.52.3
Updating 41cf32d..3d836fe
Fast-forward
 .github/dependabot.yml | 10 ++++++++++
 Cargo.lock             | 10 +++++-----
 Cargo.toml             |  2 +-
 3 files changed, 16 insertions(+), 6 deletions(-)
 create mode 100644 .github/dependabot.yml*/
