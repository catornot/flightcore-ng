use flightcore_ng_core::settings::FlightCoreSettings;
use iced::{Element, Task, widget::text};
use iced_aw::TabLabel;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    Message,
    screen::{Screen, ScreensMessage},
};

#[derive(Debug)]
pub struct ProfilesScreen {
    settings: Arc<RwLock<FlightCoreSettings>>,
}

impl ProfilesScreen {
    pub fn new(settings: Arc<RwLock<FlightCoreSettings>>) -> Self {
        Self { settings }
    }
}

impl Screen for ProfilesScreen {
    fn view(&self) -> Element<'_, Message> {
        text("Work in Progress!").size(72).into()
    }

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>> {
        Some(match message {
            _ => None?,
        })
    }

    fn label(&self) -> TabLabel {
        TabLabel::from(('🗀', "Profiles"))
    }
}
