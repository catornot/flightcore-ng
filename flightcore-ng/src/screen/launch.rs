use flightcore_ng_core::{
    launch::launch_northstar, settings::FlightCoreSettings, setup::northstar::CORE_MODS,
};
use iced::{
    Element, Length, Padding, Task,
    widget::{button, column, combo_box, container, row, space, text, tooltip, tooltip::Position},
};
use iced_aw::TabLabel;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{fs::read_to_string, sync::RwLock};

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
    version: Option<String>,
    profile_combos: combo_box::State<String>,
    did_startup: bool,
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
            version: None,
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
            servers: 0,
            players: 0,
            did_startup: false,
            settings,
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
            (true, _) => container(tooltip(
                button(text("Running").size(32)),
                "Game is running",
                Position::Bottom,
            )),
            (false, None) => container(text("Go to profiles tab").size(32)),
        }
        .padding(Padding::new(10.));

        column![
            container("").height(Length::FillPortion(10)),
            text("Northstar").size(48),
            text(self.version.as_ref().map_or_else(
                || "version: unknown".to_string(),
                |version| format!("version: {version}")
            )),
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
                            .map(ToString::to_string)
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
                Task::done(Message::Screens(ScreensMessage::NewNorthstarVersion(None)))
                    // yeah sure let's also update player count; TODO: make it based on some clock
                    .chain(Task::future(fetch_servers_stats()))
            }
            ScreensMessage::GameEnded(result) => {
                self.is_running = false;

                let task = Task::future(fetch_servers_stats());

                if let Err(err) = result {
                    task.chain(Task::done(Message::Error(err.clone())))
                } else {
                    task
                }
            }
            ScreensMessage::ServerStatsUpdate {
                players_count,
                servers_count,
            } => {
                self.servers = *servers_count;
                self.players = *players_count;
                Task::none()
            }
            ScreensMessage::NewNorthstarVersion(Some(version)) => {
                self.version = Some(version.clone());
                Task::none()
            }
            ScreensMessage::NewNorthstarVersion(None) => 'b: {
                let Some(profile) = self.profile.as_deref() else {
                    self.version = None;
                    break 'b Task::none();
                };

                Task::future(get_northstar_version(
                    Arc::clone(&self.settings),
                    profile.to_owned(),
                ))
            }

            ScreensMessage::Startup if !self.did_startup => {
                self.did_startup = true;
                Task::future(fetch_servers_stats())
                    .chain(Task::done(Message::Screens(
                        ScreensMessage::NewNorthstarVersion(None),
                    )))
                    .chain(Task::done(Message::Screens(ScreensMessage::Startup)))
            }
            _ => None?,
        })
    }

    fn label(&self) -> TabLabel {
        TabLabel::from(('⏻', "Launch"))
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
                Message::Screens(ScreensMessage::GameEnded(Ok(())))
            }
        }
    }
}

async fn get_northstar_version(
    settings: Arc<RwLock<FlightCoreSettings>>,
    profile: String,
) -> Message {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
    #[serde(rename_all = "PascalCase")]
    pub struct ModStub {
        pub version: String,
        #[serde(flatten)]
        pub _extra: HashMap<String, serde_json::Value>,
    }

    let Some(profile_path) = settings
        .read()
        .await
        .get_profile(&profile)
        .map(|profile| profile.titanfall2_path.join(&profile.name))
    else {
        return Message::Error("non existent profile".to_string());
    };

    // get version from core mods
    for modjson_path in CORE_MODS
        .iter()
        .map(|mod_path| profile_path.join(mod_path).join("mod.json"))
    {
        if let Some(version) = read_to_string(modjson_path)
            .await
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .map(|stub: ModStub| stub.version)
        {
            return Message::Screens(ScreensMessage::NewNorthstarVersion(Some(version)));
        }
    }

    Message::Error(
        "didn't find any good northstar versions on the current profile; it might be broken"
            .to_string(),
    )
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct ServerInfo {
    player_count: u32,
    #[serde(flatten)]
    pub _extra: HashMap<String, serde_json::Value>,
}

#[allow(clippy::cast_possible_truncation)]
async fn fetch_servers_stats() -> Message {
    match fetch_servers_info().await {
        Ok(servers) => Message::Screens(ScreensMessage::ServerStatsUpdate {
            servers_count: servers.len() as u32,
            players_count: servers
                .into_iter()
                .map(|info| info.player_count)
                .sum::<u32>(),
        }),
        Err(err) => Message::Error(err.to_string()),
    }
}

async fn fetch_servers_info() -> Result<Vec<ServerInfo>, eyre::Report> {
    reqwest::get("https://northstar.tf/client/servers")
        .await?
        .text()
        .await
        .map_err(eyre::Report::from)
        .and_then(|data| json5::from_str::<Vec<ServerInfo>>(&data).map_err(eyre::Report::from))
}
