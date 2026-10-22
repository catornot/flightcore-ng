use color_eyre::eyre::{Report, eyre};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub titanfall2: PathBuf,
    pub profiles: Vec<ProfileConfig>,
    pub launch_args: Vec<String>,
    pub preferred_launch: LaunchMethod,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileConfig {
    pub name: String,
    pub titanfall2_path: PathBuf,
    pub flavor: NorthstarSource,
    pub sources: Vec<String>,
    pub launch_args: Vec<String>,
    pub ignore_global_launch_args: bool,
    pub launch_method: LaunchMethod,
    pub source: ProfileConfigSource,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProfileConfigSource {
    User,
    #[default]
    Flightcoreng,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
pub enum NorthstarSource {
    Version(semver::Version),
    #[default]
    Stable,
    Nightly,
    Ion,
    Overlayed,
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

impl Config {
    pub fn add_default_profiles(&mut self) -> Option<()> {
        let titanfall2 = self.get_titanfall_path();
        if self.get_profile("R2NorthstarStable").is_none() {
            self.profiles.push(ProfileConfig {
                name: "R2NorthstarStable".to_string(),
                flavor: NorthstarSource::Stable,
                ..Default::default()
            });
        }
        if self.get_profile("R2NorthstarNightly").is_none() {
            self.profiles.push(ProfileConfig {
                name: "R2NorthstarNightly".to_string(),
                flavor: NorthstarSource::Nightly,
                ..Default::default()
            });
        }

        Some(())
    }

    #[must_use]
    pub fn get_titanfall_path(&self) -> &Path {
        self.titanfall2.as_path()
    }

    pub fn add_titanfall_path(&mut self, titanfall: PathBuf) {
        self.titanfall2.push(titanfall);
    }

    #[must_use]
    pub fn get_profiles(&self) -> &[ProfileConfig] {
        &self.profiles
    }

    #[must_use]
    pub fn get_default_profile(&self) -> Option<&ProfileConfig> {
        self.profiles.first()
    }

    #[must_use]
    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileConfig> {
        self.profiles
            .iter()
            .find(|profile| profile.name == profile_name)
    }

    pub fn get_profile_mut(&mut self, profile_name: &str) -> Option<&mut ProfileConfig> {
        self.profiles
            .iter_mut()
            .find(|profile| profile.name == profile_name)
    }

    pub fn add_profile(
        &mut self,
        profile_name: &str,
        titanfall2: Option<PathBuf>,
    ) -> Result<&mut ProfileConfig, Report> {
        if self.get_profile(profile_name).is_some() {
            return Err(eyre!("this profile already exists!"));
        }

        self.profiles.push(ProfileConfig {
            name: profile_name.to_string(),
            titanfall2_path: titanfall2.unwrap_or_else(|| self.get_titanfall_path().to_owned()),
            ..Default::default()
        });

        // the profile just got added
        Ok(self.get_profile_mut(profile_name).unwrap())
    }
}
