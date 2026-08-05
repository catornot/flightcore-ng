use color_eyre::eyre::Report;

use crate::{settings::ProfileSettings, setup::northstar::Check};

pub mod northstar;
pub mod sources;

pub async fn setup_profile(profile: &ProfileSettings) -> Result<(), Report> {
    northstar::bootstrap_northstar(profile, Check::Check).await?;

    Ok(())
}
