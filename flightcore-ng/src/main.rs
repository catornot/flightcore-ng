use flightcore_ng_core::dev::wine::{
    wine_install::{install_wine, is_wine_installed, remove_wine},
    wine_run::run_game,
};
use iced::{
    Element, Task, Theme,
    widget::{button, column, text},
};
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone)]
enum Message {
    LaunchGame(),
    Good,
    DisplayError(String),
}

#[derive(Debug, Default)]
struct FlightCore {}

fn main() -> iced::Result {
    color_eyre::install().expect("couldn't install color_eyre");
    tracing_subscriber::fmt::init();

    iced::application(|| FlightCore {}, FlightCore::update, FlightCore::view)
        .theme(FlightCore::theme)
        .run()
}

impl FlightCore {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LaunchGame() => Task::future(fun_name()),
            Message::Good => Task::none(),
            Message::DisplayError(err) => {
                error!("{err}");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("Launch").on_press(Message::LaunchGame()),
            text("Launch")
        ]
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Oxocarbon
    }
}

async fn fun_name() -> Message {
    if !is_wine_installed() {
        info!("installing wine prefix");

        // todo add progress bar
        if let Err(err) = install_wine().await {
            _ = remove_wine().await;
            return Message::DisplayError(err.to_string());
        }
    }
    info!(
        "launching the game at /home/catornot/.local/share/Steam/steamapps/common/Titanfall2/NorthstarLauncher.exe"
    );
    match run_game(
        &PathBuf::from(
            "/home/catornot/.local/share/Steam/steamapps/common/Titanfall2/NorthstarLauncher.exe",
        ),
        &[],
    )
    .await
    {
        Ok(()) => Message::Good,
        Err(err) => Message::DisplayError(err.to_string()),
    }
}
