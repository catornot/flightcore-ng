use flightcore_ng_core::settings::FlightCoreSettings;
use iced::{
    Element, Padding, Task,
    widget::{column, container, scrollable, text, tooltip},
};
use iced_aw::{Card, TabLabel};
use std::sync::Arc;
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

use crate::{
    Message,
    screen::{Screen, ScreensMessage},
};

#[derive(Debug)]
pub struct ProfilesScreen {
    settings: Arc<RwLock<FlightCoreSettings>>,
    cached_profiles: Vec<String>,
    lock: Option<OwnedRwLockWriteGuard<FlightCoreSettings>>,
    focused: bool, // TODO: don't handle such state here?
}

impl ProfilesScreen {
    pub const fn new(settings: Arc<RwLock<FlightCoreSettings>>) -> Self {
        Self {
            settings,
            lock: None,
            cached_profiles: Vec::new(),
            focused: false,
        }
    }
}

impl Screen for ProfilesScreen {
    fn view(&self) -> Element<'_, Message> {
        self.lock.as_ref().map_or_else(
            || no_lock_view(&self.cached_profiles),
            |lock| {
                container(scrollable(column(vec![text("e").into()])))
                    .padding(Padding::horizontal(Padding::ZERO, 5.))
                    .into()
            },
        )
    }

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>> {
        Some(match message {
            ScreensMessage::SwitchedTo => {
                self.focused = true;
                Task::future(acquire_lock(Arc::clone(&self.settings))).map(Message::Screens)
            }
            ScreensMessage::SwitchedFrom => {
                self.focused = false;
                self.lock = None;
                Task::none()
            }
            ScreensMessage::AcquiredLock(lock) if self.focused => {
                if let Some(acquired_lock) = lock.take().and_then(|lock| Arc::try_unwrap(lock).ok())
                {
                    self.cached_profiles = build_profile_cache(&acquired_lock);
                    self.lock = Some(acquired_lock);
                    Task::none()
                } else {
                    Task::done(Message::Error("Failed to Acquire lock".to_string())).chain(
                        Task::done(Message::Screens(ScreensMessage::SwitchActiveScreen(0))),
                    )
                }
            }
            _ => None?,
        })
    }

    fn label(&self) -> TabLabel {
        TabLabel::from(('🗀', "Profiles"))
    }
}

fn no_lock_view(cached_profiles: &[String]) -> Element<'_, Message> {
    match cached_profiles.len() {
        0 => text("The application is busy, cannot display profiles at the moment")
            .size(72)
            .into(),
        1.. => container(tooltip(
            scrollable(column(cached_profiles.iter().map(|profile| {
                Card::new(text(profile), text(profile).size(32)).into()
            }))),
            "The application is busy, cannot display profiles at the moment",
            iced::widget::tooltip::Position::FollowCursor,
        ))
        .padding(Padding::horizontal(Padding::ZERO, 5.))
        .into(),
    }
}

fn build_profile_cache(settings: &FlightCoreSettings) -> Vec<String> {
    settings
        .get_profiles()
        .iter()
        .map(|profile| profile.name.clone())
        .collect()
}

async fn acquire_lock(settings: Arc<RwLock<FlightCoreSettings>>) -> ScreensMessage {
    ScreensMessage::AcquiredLock(Some(Arc::new(settings.write_owned().await)))
}
