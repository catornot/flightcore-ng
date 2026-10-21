use color_eyre::eyre::{Report, WrapErr, eyre};
use eyre::ContextCompat;
use nix_compat::flakeref::FlakeRef;
use reqwest::Url;
use snix_eval::Value;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use crate::{setup::sources::SourceImpl, snix_ext::CloneableFlakeRef};

#[derive(Debug, Clone)]
pub struct Northstar {
    pub launcher: CloneableFlakeRef,
    pub mods: CloneableFlakeRef,
    pub discordrpc: CloneableFlakeRef,
    pub plugins: CloneableFlakeRef,
}

pub enum NorthstarSource {}

impl SourceImpl for Northstar {
    fn get_name(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.launcher.hash(&mut hasher);
        self.mods.hash(&mut hasher);
        self.discordrpc.hash(&mut hasher);
        self.plugins.hash(&mut hasher);

        hasher.finish().to_string()
    }

    fn install_path(&self, _profile_path: &Path) -> Option<PathBuf> {
        None
    }

    async fn instantiate(&self, _store_path: &Path) -> Result<(), Report> {
        Ok(())
    }

    async fn is_instantiated(&self, _store_path: &Path) -> bool {
        true
    }
}

impl Northstar {
    pub fn try_from_value<'a>(source: &Value) -> Result<Option<Self>, Report> {
        let source = match source.to_attrs() {
            Ok(source) => source,
            Err(err) => return Err(eyre!(err.to_string())),
        };
        if source
            .select("_type")
            .map(|ty| ty.to_string() == "northstar")
            .unwrap_or_default()
        {
            return Ok(None);
        }

        // TODO: figure out if discord and plugins should be auto filled
        let repo = Self {
            launcher: source
                .select("launcher")
                .map(|value| value.to_string().parse::<FlakeRef>())
                .wrap_err_with(|| eyre!("failed to find flake ref for launcher"))?
                .wrap_err_with(|| eyre!("failed to parse the launcher flake ref as a flake ref"))?
                .into(),
            mods: source
                .select("mods")
                .map(|value| value.to_string().parse::<FlakeRef>())
                .wrap_err_with(|| eyre!("failed to find flake ref for mods"))?
                .wrap_err_with(|| eyre!("failed to parse the mods flake ref as a flake ref"))?
                .into(),
            discordrpc: invert_result_option(
                source
                    .select("discordrpc")
                    .map(|value| value.to_string().parse::<FlakeRef>()),
            )
            .map_err(|err| eyre!(err.to_string()))
            .wrap_err_with(|| eyre!("wrong type for discordrpc"))?
            .map(|flakeref| CloneableFlakeRef(flakeref))
            .unwrap_or_else(|| get_github_flake_ref("R2Northstar", "DiscordRPC").into()),
            plugins: invert_result_option(
                source
                    .select("plugins")
                    .map(|value| value.to_string().parse::<FlakeRef>()),
            )
            .map_err(|err| eyre!(err.to_string()))
            .wrap_err_with(|| eyre!("wrong type for plugins"))?
            .map(|flakeref| CloneableFlakeRef(flakeref))
            .unwrap_or_else(|| get_github_flake_ref("R2Northstar", "NorthstarPlugins").into()),
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

fn get_repo_flake_ref(sha: Option<String>, repo: &'static str) -> Result<FlakeRef, Report> {
    Ok(FlakeRef::Git {
        all_refs: false,
        export_ignore: false,
        keytype: None,
        public_key: None,
        public_keys: None,
        r#ref: None,
        rev: sha,
        shallow: false,
        submodules: true,
        url: repo.try_into()?,
        verify_commit: false,
    })
}

fn get_github_flake_ref(owner: &str, repo: &str) -> FlakeRef {
    FlakeRef::GitHub {
        owner: owner.to_string(),
        repo: repo.to_string(),
        host: None,
        keytype: None,
        public_key: None,
        public_keys: None,
        r#ref: None,
        rev: None,
    }
}
