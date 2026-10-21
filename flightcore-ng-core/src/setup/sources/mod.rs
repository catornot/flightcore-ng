use color_eyre::eyre::{Report, WrapErr, eyre};
use eyre::ContextCompat;
use snix_eval::{EvalIO, EvalMode, FileType, NixAttrs};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::{
    local_dir,
    settings::{Config, LaunchMethod, NorthstarSource, ProfileConfig},
    setup::sources::{northstar::Northstar, repos::Repo},
};

mod mods;
mod northstar;
mod packages;
mod repos;

const EVAL_CODE: &str = r#"
{
    config = (import ./config.nix {}) // (import ./generated-config.nix {});
}
"#;

#[derive(Debug, Clone)]
pub enum Source {
    Mod,
    Package,
    Repo(Repo),
    Northstar(Northstar),
    LockedSource(()),
}

pub struct Store {
    store_path: PathBuf,
    eval_path: PathBuf,
    // TODO: find a better data structure for this
    sources: Vec<(String, Source)>,
    pub config: Config,
}

struct FlightCoreEvalIO {
    store_path: PathBuf,
    eval_path: PathBuf,
}

pub trait SourceImpl {
    fn get_name(&self) -> String;

    fn install_path(&self, profile_path: &Path) -> Option<PathBuf>;

    async fn instantiate(&self, store_path: &Path) -> Result<(), Report>;

    async fn is_instantiated(&self, store_path: &Path) -> bool {
        self.instantiate(store_path).await.is_ok()
    }
}

impl Store {
    pub fn new() -> Self {
        let eval_path = local_dir().expect("bruh how");
        let store_path = eval_path.join("store");
        _ = std::fs::create_dir_all(&store_path);

        let store = Self {
            store_path,
            eval_path,
            sources: Vec::new(),
            config: Default::default(),
        };

        store
    }

    pub(super) fn store_path(&self) -> &Path {
        &self.store_path
    }

    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileConfig> {
        self.config
            .profiles
            .iter()
            .find(|profile| profile.name == profile_name)
    }

    pub fn get_source(&self, name: &str) -> Option<&Source> {
        self.sources
            .iter()
            .find_map(|(src_name, src)| src_name.eq(name).then_some(src))
    }

    pub fn evaluate_config(&self) -> Result<(), Report> {
        _ = fs::File::create_new(self.eval_path.join("config.nix"));
        _ = fs::File::create_new(self.eval_path.join("generated-config.nix"));

        let result = snix_eval::EvaluationBuilder::new(Box::new(FlightCoreEvalIO {
            store_path: self.store_path.clone(),
            eval_path: self.eval_path.clone(),
        }) as Box<dyn EvalIO>)
        .enable_import()
        .mode(EvalMode::Strict)
        .build()
        .evaluate(EVAL_CODE, Some(self.eval_path.clone()));

        if let Some(err) = result.errors.iter().fold(None::<Report>, |acc, err| {
            if let Some(acc) = acc {
                Some(acc.wrap_err(eyre!(err.fancy_format_str())))
            } else {
                Some(eyre!(err.fancy_format_str()))
            }
        }) {
            return Err(err);
        }

        let Some(value) = result.value else {
            return Ok(());
        };

        let config = value
            .to_attrs()
            .map_err(|err| eyre!("internal evaluation errors report please : {err}"))?
            .iter()
            .find_map(|(key, value)| (*key == "config").then_some(value))
            .wrap_err_with(|| eyre!("internal evaluation errors report please"))?
            .to_attrs()
            .map_err(|err| eyre!("evaluation errors in merge report please : {err}"))?;

        info!(
            "we got {} {}",
            config
                .iter()
                .map(|(_, value)| value.to_string())
                .collect::<String>(),
            value.type_of()
        );

        Ok(())
    }
}

impl EvalIO for FlightCoreEvalIO {
    fn path_exists(&self, path: &Path) -> std::io::Result<bool> {
        Ok(path
            .parent()
            .map(|parent| self.store_path == parent || self.eval_path == parent)
            .unwrap_or_default()
            && path.exists())
    }

    fn open(&self, path: &Path) -> std::io::Result<Box<dyn std::io::Read>> {
        std::fs::File::open(path).map(|file| Box::new(file) as Box<dyn std::io::Read>)
    }

    fn file_type(&self, path: &Path) -> std::io::Result<FileType> {
        if path.is_symlink() {
            Ok(FileType::Symlink)
        } else if path.is_file() {
            Ok(FileType::Regular)
        } else if path.is_dir() {
            Ok(FileType::Directory)
        } else {
            Ok(FileType::Unknown)
        }
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<(bytes::Bytes, FileType)>> {
        let mut acc = Vec::new();
        for read in path.read_dir()?.filter_map(|i| i.ok()) {
            acc.push((fs::read(read.path())?.into(), self.file_type(&read.path())?));
        }
        Ok(acc)
    }

    fn import_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        Ok(path.to_path_buf())
    }

    fn get_env(&self, key: &std::ffi::OsStr) -> Option<std::ffi::OsString> {
        std::env::var_os(key)
    }
}

fn evaluate_config(config: &NixAttrs) -> Result<Config, Report> {
    let titanfall2 = evaluate_titanfall2_path(config, "", "config")
        .wrap_err_with(|| eyre!("no titanfall2 path configured"))??;
    Ok(Config {
        profiles: evaluate_profiles(config, &titanfall2)?,
        launch_args: evaluate_launch_args(config, "", "config")?,
        preferred_launch: evaluate_launch_method(config),
        titanfall2,
    })
}

fn evaluate_sources(config: &NixAttrs) -> Vec<Source> {
    todo!()
}

fn evaluate_profiles(
    config: &NixAttrs,
    global_titanfall: &Path,
) -> Result<Vec<ProfileConfig>, Report> {
    let Some(profiles) = config
        .select("profiles")
        .map(|value| value.to_list().map_err(|err| eyre!(err.to_string())))
    else {
        return Ok(Vec::new());
    };
    let profiles = profiles
        .wrap_err_with(|| eyre!("profiles aren't isn't a list"))?
        .into_inner();
    let profiles = profiles
        .iter()
        .map(|value| value.to_attrs().map_err(|err| eyre!(err.to_string())))
        .collect::<Result<Vec<_>, _>>()
        .wrap_err_with(|| eyre!("profiles isn't a list of attrs"))?;
    profiles
        .iter()
        .count()
        .eq(&0)
        .then_some(())
        .ok_or_else(|| eyre!("empty profiles"))?;

    let mut acc = Vec::new();
    for profile in profiles {
        let name = profile
            .select("name")
            .wrap_err_with(|| eyre!("profile is missing a name"))?
            .to_string();

        acc.push(ProfileConfig {
            titanfall2_path: evaluate_titanfall2_path(&profile, "profile ", &name)
                .unwrap_or_else(|| Ok(global_titanfall.to_path_buf()))?,
            flavor: evaluate_flavor(&profile, &name)?,
            sources: evaluate_sources_ref(&profile, &name)?,
            launch_args: evaluate_launch_args(&profile, "profile ", &name)?,
            ignore_global_launch_args: profile
                .select("ignore-global-launch-args")
                .and_then(|value| value.as_bool().ok())
                .unwrap_or_default(),
            launch_method: evaluate_launch_method(&profile),
            name,
        });
    }

    Ok(acc)
}

fn evaluate_titanfall2_path(
    config: &NixAttrs,
    profile: &str,
    name: &str,
) -> Option<Result<PathBuf, Report>> {
    config.select("titanfall2-path").map(|value| {
        value
            .to_path()
            .map_err(|err| eyre!(err.to_string()))
            .wrap_err_with(|| eyre!("{profile}{name} has a poorly typed path to titanfall2"))
            .map(|p| *p)
    })
}

fn evaluate_flavor(profile: &NixAttrs, name: &String) -> Result<NorthstarSource, Report> {
    let flavor = profile
        .select("flavor")
        .wrap_err_with(|| eyre!("profile {name} is missing the flavor indicator of northstar"))?
        .to_attrs()
        .map_err(|err| eyre!(err.to_string()))
        .wrap_err_with(|| {
            eyre!("profile {name} has poorly formatted typed flavor; use builtins.mkFlavor")
        })?;

    Ok(
        match flavor
            .select("_type")
            .map(|value| value.to_string().to_lowercase())
        {
            Some(ty) if ty == "version" => NorthstarSource::Version(
                flavor
                    .select("version")
                    .map(|value| value.to_string().to_lowercase())
                    .and_then(|version| version.parse().ok())
                    .wrap_err_with(|| eyre!("bad version or no version"))?,
            ),
            Some(ty) if ty == "stable" => NorthstarSource::Stable,
            Some(ty) if ty == "nightly" => NorthstarSource::Nightly,
            Some(ty) if ty == "ion" => NorthstarSource::Ion,
            Some(ty) if ty == "overlayed" => NorthstarSource::Overlayed,
            _ => {
                return Err(eyre!("bad flavor!")).wrap_err_with(|| {
                    eyre!("profile {name} has a bad typed flavor; use builtins.mkFlavor")
                });
            }
        },
    )
}

fn evaluate_launch_method(profile: &NixAttrs) -> LaunchMethod {
    profile
        .select("launch-method")
        .map(|value| match value.to_string().to_lowercase().as_str() {
            "any" => LaunchMethod::Any,
            "steam" => LaunchMethod::Steam,
            "wine" => LaunchMethod::Wine,
            "direct" => LaunchMethod::Direct,
            _ => LaunchMethod::default(),
        })
        .unwrap_or_default()
}

fn evaluate_sources_ref(profile: &NixAttrs, name: &str) -> Result<Vec<String>, Report> {
    Ok(profile
        .select("sources")
        .map(|list| {
            list.to_list()
                .map_err(|err| eyre!(err.to_string()))
                .wrap_err_with(|| eyre!("profile {name} has sources that aren't a list"))
                .map(|list| list.into_inner())
        })
        .unwrap_or_else(|| Ok(Vec::new()))?
        .into_iter()
        .map(|value| value.to_str().map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| eyre!(err.to_string()))
        .wrap_err_with(|| {
            eyre!("profile {name} has sources that properly setup; use builtins.mkSourceRef")
        })?)
}

fn evaluate_launch_args(
    config: &NixAttrs,
    profile: &str,
    name: &str,
) -> Result<Vec<String>, Report> {
    Ok(config
        .select("launch-args")
        .map(|list| {
            list.to_list()
                .map_err(|err| eyre!(err.to_string()))
                .wrap_err_with(|| eyre!("{profile}{name} has launch-args that aren't a list"))
                .map(|list| list.into_inner())
        })
        .unwrap_or_else(|| Ok(Vec::new()))?
        .into_iter()
        .map(|value| value.to_str().map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| eyre!(err.to_string()))
        .wrap_err_with(|| eyre!("{profile}{name} has launch-args that aren't strings"))?)
}

// fn evaluate_flavor(flavor: NixAttrs) -> Result<NorthstarSource, Report> {
//     let source = match flavor
//         .select("_type")
//         .map(|value| value.to_string().to_lowercase())
//     {
//         Some(ty) if ty == "version" => NorthstarSource::Version(
//             flavor
//                 .select("version")
//                 .map(|value| value.to_string().to_lowercase())
//                 .and_then(|version| version.parse().ok())
//                 .wrap_err_with(|| eyre!("bad version or no version"))?,
//         ),
//         Some(ty) if ty == "stable" => NorthstarSource::Stable,
//         Some(ty) if ty == "nightly" => NorthstarSource::Nightly,
//         Some(ty) if ty == "ion" => NorthstarSource::Ion,
//         Some(ty) if ty == "overlayed" => NorthstarSource::Overlayed,
//         _ => return Err(eyre!("bad flavor!")),
//     };

//     Ok(source)
// }
