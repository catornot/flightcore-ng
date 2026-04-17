use color_eyre::eyre::{Report, WrapErr};
use eyre::eyre;
use reqwest::Url;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tokio::fs;

use crate::local_dir;

pub static SETTINGS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    local_dir()
        .expect("local dir should not be failing")
        .join("settings.ron")
});
pub static PREATTY_CONFIG: LazyLock<PrettyConfig> = LazyLock::new(PrettyConfig::new);

#[derive(Debug, Clone)]
pub struct FlightCoreSettings {
    pub settings: Settings,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Settings {
    pub titanfall2: Vec<PathBuf>,
    profiles: Vec<ProfileSettings>,
    pub sources: Vec<Source>,
    pub launch_args: Vec<String>,
    pub preferred_launch: LaunchMethod,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProfileSettings {
    pub name: String,
    pub titanfall2_path: PathBuf,
    pub northstar: NorthstarSource,
    pub sources: Vec<Source>,
    pub launch_args: Vec<String>,
    pub ignore_global_launch_args: bool,
    pub launch_method: LaunchMethod,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub enum Source {
    Launcher(LauncherSource),
    CoreMods(CoreModsSource),
    DiscordRPC(DiscordRPCSource),
    Mod(ModInfo),
    ModRepo(Url),
    Package(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
pub struct ModInfo {}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
pub enum NorthstarSource {
    Version(semver::Version),
    #[default]
    Stable,
    Nightly,
    Ion,
    Overlayed,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub enum LauncherSource {
    FromCommit(String),
    Version(semver::Version),
    Path(PathBuf),
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub enum CoreModsSource {
    FromCommit(String),
    Version(semver::Version),
    Path(PathBuf),
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub enum DiscordRPCSource {
    FromCommit(String),
    Version(semver::Version),
    Path(PathBuf),
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, Default, Hash, PartialEq, PartialOrd, Ord, Eq,
)]
pub enum LaunchMethod {
    #[default]
    Any,
    Steam,
    Wine,
    Direct,
}

impl Source {
    pub fn as_launcher(&self) -> Option<&LauncherSource> {
        match self {
            Self::Launcher(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_core_mods(&self) -> Option<&CoreModsSource> {
        match self {
            Self::CoreMods(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_discord_rpc(&self) -> Option<&DiscordRPCSource> {
        match self {
            Self::DiscordRPC(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_mod(&self) -> Option<&ModInfo> {
        match self {
            Self::Mod(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_mod_repo(&self) -> Option<&Url> {
        match self {
            Self::ModRepo(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_package(&self) -> Option<&String> {
        match self {
            Self::Package(inner) => Some(inner),
            _ => None,
        }
    }
}

impl FlightCoreSettings {
    pub async fn load() -> Result<FlightCoreSettings, Report> {
        if !SETTINGS_PATH.exists() {
            fs::write(
                SETTINGS_PATH.as_path(),
                &ron::to_string(&Settings::default())?,
            )
            .await
            .wrap_err_with(|| {
                eyre!(
                    "couldn't create the settings file : {}",
                    SETTINGS_PATH.display()
                )
            })?;
        }

        Ok(FlightCoreSettings {
            settings: ron::from_str(
                &fs::read_to_string(SETTINGS_PATH.as_path())
                    .await
                    .wrap_err_with(|| {
                        eyre!(
                            "couldn't read the settings file : {}",
                            SETTINGS_PATH.display()
                        )
                    })?,
            )?,
        })
    }

    pub fn add_default_profiles(&mut self) -> Option<()> {
        let titanfall2 = self.get_default_titanfall_path()?.to_owned();
        if self.get_profile("R2NorthstarStable").is_none() {
            self.settings.profiles.push(ProfileSettings {
                name: "R2NorthstarStable".to_string(),
                titanfall2_path: titanfall2.clone(),
                northstar: NorthstarSource::Stable,
                ..Default::default()
            });
        }
        if self.get_profile("R2NorthstarNightly").is_none() {
            self.settings.profiles.push(ProfileSettings {
                name: "R2NorthstarNightly".to_string(),
                titanfall2_path: titanfall2.clone(),
                northstar: NorthstarSource::Nightly,
                ..Default::default()
            });
        }
        // bruh who even uses rc s lol
        // if self.get_profile("R2NorthstarRC").is_none() {}

        Some(())
    }

    pub fn get_titanfall_path_from_profile(&self, profile: &str) -> Option<&Path> {
        self.get_profile(profile)
            .map(|profile| profile.titanfall2_path.as_path())
    }

    pub fn get_default_titanfall_path(&self) -> Option<&Path> {
        self.settings.titanfall2.first().map(|path| path.as_path())
    }

    pub fn add_titanfall_path(&mut self, titanfall: PathBuf) {
        self.settings.titanfall2.push(titanfall);
    }

    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileSettings> {
        self.settings
            .profiles
            .iter()
            .find(|profile| profile.name == profile_name)
    }

    pub fn get_profile_mut(&mut self, profile_name: &str) -> Option<&mut ProfileSettings> {
        self.settings
            .profiles
            .iter_mut()
            .find(|profile| profile.name == profile_name)
    }

    pub fn add_profile(
        &mut self,
        profile_name: &str,
        titanfall2: Option<PathBuf>,
    ) -> Result<&mut ProfileSettings, Report> {
        if self.get_profile(profile_name).is_some() {
            return Err(eyre!("this profile already exists!"));
        }

        self.settings.profiles.push(ProfileSettings {
            name: profile_name.to_string(),
            titanfall2_path: titanfall2
                .or_else(|| self.get_default_titanfall_path().map(ToOwned::to_owned))
                .ok_or_else(|| eyre!("no default titanfall path configured"))?,
            ..Default::default()
        });

        // the profile just got added
        Ok(self.get_profile_mut(profile_name).unwrap())
    }

    pub async fn save(&self) -> Result<(), Report> {
        fs::write(
            SETTINGS_PATH.as_path(),
            ron::ser::to_string_pretty(&self.settings, PREATTY_CONFIG.clone())?,
        )
        .await
        .wrap_err("couldn't save settings")
    }
}

impl Drop for FlightCoreSettings {
    fn drop(&mut self) {
        _ = ron::ser::to_string_pretty(&self.settings, PREATTY_CONFIG.clone())
            .map(|settings| _ = std::fs::write(SETTINGS_PATH.as_path(), settings));
    }
}
