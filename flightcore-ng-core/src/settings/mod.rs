use color_eyre::eyre::{Report, WrapErr};
use eyre::eyre;
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
    pub profiles: Vec<ProfileSettings>,
    pub mods: Vec<ModInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProfileSettings {}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModInfo {}

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

    pub fn get_titanfall_path_from_profile(&self, profile: &str) -> Option<&Path> {
        self.settings.titanfall2.first().map(|path| path.as_path())
    }

    pub fn get_default_titanfall_path(&self) -> Option<&Path> {
        self.settings.titanfall2.first().map(|path| path.as_path())
    }

    pub fn add_titanfall_path(&mut self, titanfall: PathBuf) {
        self.settings.titanfall2.push(titanfall);
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
