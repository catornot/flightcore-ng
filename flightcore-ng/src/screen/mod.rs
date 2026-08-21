use std::sync::Arc;

use flightcore_ng_core::settings::FlightCoreSettings;
use iced::{
    Element, Length, Task,
    widget::{Row, button, column},
};
use tokio::sync::RwLock;

use crate::{Message, screen::launch::LaunchScreen};

pub mod launch;

#[derive(Debug)]
pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
    active: usize,
}

impl Screens {
    pub fn new(settings: Arc<RwLock<FlightCoreSettings>>) -> Self {
        Self {
            screens: vec![Box::new(LaunchScreen::new(Arc::clone(&settings)))],
            active: 0,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            Row::from_vec(
                self.screens
                    .iter()
                    .map(|screen| screen.name())
                    .enumerate()
                    .map(|(index, name)| button(name)
                        .on_press(Message::Screens(ScreensMessage::SwitchActiveScren(index)))
                        .into())
                    .collect(),
            )
            .spacing(30)
            .width(Length::Fill),
            self.screens
                .get(self.active)
                .unwrap_or_else(|| self.screens.first().expect("must have at least one screen"))
                .view()
        ]
        .into()
    }

    pub fn update(&mut self, message: ScreensMessage) -> Task<Message> {
        match message {
            ScreensMessage::SwitchActiveScren(index) => {
                if index < self.screens.len() {
                    self.active = index;
                    Task::none()
                } else {
                    Task::done(Message::Error(format!("Couldn't switch to screen {index}")))
                }
            }
            mut message => self
                .screens
                .iter_mut()
                .find_map(|screen| screen.update(&mut message))
                .unwrap_or_else(Task::none),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScreensMessage {
    SwitchActiveScren(usize),
    SwitchProfile(String),
    LaunchGame { profile: String },
    GameEnded(Result<(), String>),
    ServersUpdated(u32),
    PlayersUpdated(u32),
}

pub trait Screen: std::fmt::Debug {
    fn view(&self) -> Element<'_, Message>;

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>>;

    fn name(&self) -> &'static str;
}
