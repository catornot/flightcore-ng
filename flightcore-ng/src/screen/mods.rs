use flightcore_ng_core::setup::sources::Store;
use iced::{Element, Task, widget::text};
use iced_aw::TabLabel;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    Message,
    screen::{Screen, ScreensMessage},
};

#[derive(Debug)]
pub struct ModsScreen {
    store: Arc<RwLock<Store>>,
}

impl ModsScreen {
    pub const fn new(store: Arc<RwLock<Store>>) -> Self {
        Self { store }
    }
}

impl Screen for ModsScreen {
    fn view(&self) -> Element<'_, Message> {
        text("Work in Progress!").size(72).into()
    }

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>> {
        Some(match message {
            _ => None?,
        })
    }

    fn label(&self) -> TabLabel {
        TabLabel::from(('Ⓘ', "Mods"))
    }
}
