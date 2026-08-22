use color_eyre::eyre::Report;
use tracing::info;

use crate::{settings::ProfileSettings, setup::northstar::Check};

pub mod northstar;
pub mod sources;

pub async fn setup_profile(profile: &ProfileSettings) -> Result<(), Report> {
    info!("setting up profile {}", profile.name);

    northstar::bootstrap_northstar(profile, Check::Check).await?;

    Ok(())
}
