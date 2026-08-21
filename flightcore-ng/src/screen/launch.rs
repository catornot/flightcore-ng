use flightcore_ng_core::{launch::launch_northstar, settings::FlightCoreSettings};
use iced::{
    Element, Length, Padding, Task,
    widget::{button, column, combo_box, container, row, space, text},
};
use semver::Version;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    Message,
    screen::{Screen, ScreensMessage},
};

#[derive(Debug)]
pub struct LaunchScreen {
    settings: Arc<RwLock<FlightCoreSettings>>,
    profile: Option<String>,
    tip: String,
    launch_args: String,
    is_running: bool,
    version: semver::Version,
    profile_combos: combo_box::State<String>,
    servers: u32,
    players: u32,
}

impl LaunchScreen {
    pub fn new(settings: Arc<RwLock<FlightCoreSettings>>) -> Self {
        Self {
            profile: settings.try_read().ok().and_then(|settings| {
                settings
                    .get_default_profile()
                    .map(|profile| profile.name.clone())
            }),
            tip: String::new(),
            launch_args: String::new(),
            is_running: false,
            version: Version::new(0, 0, 0),
            profile_combos: combo_box::State::new(
                settings
                    .try_read()
                    .ok()
                    .map(|settings| {
                        settings
                            .get_profiles()
                            .iter()
                            .map(|profile| profile.name.clone())
                            .collect()
                    })
                    .unwrap_or_default(),
            ),
            settings,
            servers: 0,
            players: 0,
        }
    }
}

impl Screen for LaunchScreen {
    fn view(&self) -> Element<'_, Message> {
        let launch_button = match (self.is_running, self.profile.as_ref()) {
            (false, Some(profile)) => {
                container(button(text("Launch").size(32)).on_press_with(|| {
                    Message::Screens(ScreensMessage::LaunchGame {
                        profile: profile.clone(),
                    })
                }))
            }
            (true, _) => container(text("Running").size(32)),
            (false, None) => container(text("Go to profiles tab").size(32)),
        }
        .padding(Padding::new(10.));

        column![
            container("").height(Length::FillPortion(10)),
            text("Northstar").size(48),
            text(format!("version: {}", self.version)),
            text(format!(
                "{} players, {} servers",
                self.players, self.servers
            )),
            row![
                launch_button,
                row![
                    container(text("Profile: ")).padding(Padding::ZERO.top(4.)),
                    combo_box(
                        &self.profile_combos,
                        "Profile",
                        self.profile.as_ref(),
                        |profile| Message::Screens(ScreensMessage::SwitchProfile(profile))
                    ),
                    space().width(Length::FillPortion(1))
                ]
                .padding(Padding::ZERO.top(20.))
            ],
            container(text(self.tip.clone())).padding(Padding::ZERO.bottom(50))
        ]
        .padding(Padding::ZERO.left(30))
        .height(Length::Fill)
        .into()
    }

    fn update(&mut self, message: &mut ScreensMessage) -> Option<Task<Message>> {
        Some(match message {
            ScreensMessage::LaunchGame { profile } => {
                if let Ok(_lock) = self.settings.try_read() {
                    self.is_running = true;
                    Task::future(launch_northstar_thunk(
                        Arc::clone(&self.settings),
                        profile.clone(),
                        self.launch_args
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    ))
                } else {
                    Task::done(Message::Notification("flightcore-ng is busy".to_string()))
                }
            }
            ScreensMessage::SwitchProfile(new_profile) => {
                if let Some(profile) = self.profile.as_mut() {
                    std::mem::swap(profile, new_profile);
                } else {
                    _ = self.profile.replace(new_profile.clone());
                }
                Task::none()
            }
            ScreensMessage::GameEnded(result) => {
                self.is_running = false;

                let tasks = Task::future(fetch_players()).chain(Task::future(fetch_servers()));

                if let Err(err) = result {
                    tasks.chain(Task::done(Message::Error(err.to_string())))
                } else {
                    tasks
                }
            }
            ScreensMessage::ServersUpdated(count) => {
                self.servers = *count;
                Task::none()
            }
            ScreensMessage::PlayersUpdated(count) => {
                self.players = *count;
                Task::none()
            }
            _ => None?,
        })
    }

    fn name(&self) -> &'static str {
        "Launch"
    }
}

async fn launch_northstar_thunk(
    settings: Arc<RwLock<FlightCoreSettings>>,
    profile: String,
    launch_args: Vec<String>,
) -> Message {
    let lock = settings.read().await;
    let launch_result = launch_northstar(&lock, &profile, launch_args).await;
    drop(lock);

    match launch_result {
        Err(err) => Message::Screens(ScreensMessage::GameEnded(Err(err
            .wrap_err(format!("failed to step profile : {profile}"))
            .to_string()))),
        Ok(runner) => {
            if let Err(err) = runner.await {
                Message::Screens(ScreensMessage::GameEnded(Err(err
                    .wrap_err(format!("game failed while running profile : {profile}"))
                    .to_string())))
            } else {
                Message::Notification("Game finished running".to_string())
            }
        }
    }
}

async fn fetch_players() -> Message {
    Message::Screens(ScreensMessage::PlayersUpdated(0))
}

async fn fetch_servers() -> Message {
    Message::Screens(ScreensMessage::ServersUpdated(0))
}
