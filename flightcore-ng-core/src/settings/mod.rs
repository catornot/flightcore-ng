use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    pub titanfall2: PathBuf,
    pub profiles: Vec<ProfileConfig>,
    pub launch_args: Vec<String>,
    pub preferred_launch: LaunchMethod,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProfileConfig {
    pub name: String,
    pub titanfall2_path: PathBuf,
    pub flavor: NorthstarSource,
    pub sources: Vec<String>,
    pub launch_args: Vec<String>,
    pub ignore_global_launch_args: bool,
    pub launch_method: LaunchMethod,
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
