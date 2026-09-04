use std::sync::Arc;

use flightcore_ng_core::settings::FlightCoreSettings;
use iced::{
    Color, Element, Length, Task,
    alignment::{Horizontal, Vertical},
    widget::{column, container, space},
};
use iced_aw::{TabLabel, widgets::TabBar};
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

use crate::{
    Message,
    screen::{launch::LaunchScreen, mods::ModsScreen, profiles::ProfilesScreen},
};

pub mod launch;
pub mod mods;
pub mod profiles;

#[derive(Debug)]
pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
    active: usize,
}

impl Screens {
    pub fn new(settings: &Arc<RwLock<FlightCoreSettings>>) -> Self {
        Self {
            screens: vec![
                Box::new(LaunchScreen::new(Arc::clone(settings))),
                Box::new(ModsScreen::new(Arc::clone(settings))),
                Box::new(ProfilesScreen::new(Arc::clone(settings))),
            ],
            active: 0,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let bar = self.create_bar();

        column![
            bar,
            space().height(Length::Fixed(10.)),
            self.screens
                .get(self.active)
                .unwrap_or_else(|| self.screens.first().expect("must have at least one screen"))
                .view()
        ]
        .into()
    }

    fn create_bar(&self) -> Element<'_, Message> {
        let bar = TabBar::with_tab_labels(
            self.screens
                .iter()
                .map(|screen| screen.label())
                .enumerate()
                .collect(),
            |tab_id| Message::Screens(ScreensMessage::SwitchActiveScreen(tab_id)),
        )
        .tab_width(Length::Shrink)
        .style(|theme, status| iced_aw::tab_bar::Style {
            text_color: Color::BLACK,
            ..iced_aw::style::tab_bar::primary(theme, status)
        })
        .spacing(25)
        .set_active_tab(&self.active)
        .width(Length::Shrink);

        container(bar)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .width(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: ScreensMessage) -> Task<Message> {
        match message {
            ScreensMessage::SwitchActiveScreen(index) => {
                if index < self.screens.len() && index != self.active {
                    let maybe_task =
                        self.screens[self.active].update(&mut ScreensMessage::SwitchedFrom);
                    self.active = index;
                    self.screens[index]
                        .update(&mut ScreensMessage::SwitchedTo)
                        .map_or_else(Task::none, |task| {
                            if let Some(inner) = maybe_task {
                                inner.chain(task)
                            } else {
                                task
                            }
                        })
                } else if index == self.active {
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
    SwitchActiveScreen(usize),
    SwitchedTo,
    SwitchedFrom,
    SwitchProfile(String),
    LaunchGame {
        profile: String,
    },
    GameEnded(Result<(), String>),
    ServerStatsUpdate {
        players_count: u32,
        servers_count: u32,
    },
    Startup,
    NewNorthstarVersion(Option<String>),
    AcquiredLock(Option<Arc<OwnedRwLockWriteGuard<FlightCoreSettings>>>),
}

pub trait Screen: std::fmt::Debug {
    fn view(&self) -> Element<'_, Message>;

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>>;

    fn label(&self) -> TabLabel;
}
