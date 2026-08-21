use std::sync::Arc;

use flightcore_ng_core::settings::FlightCoreSettings;
use iced::{Element, Task, Theme};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::screen::{Screens, ScreensMessage};

mod screen;

#[derive(Debug, Clone)]
enum Message {
    Notification(String),
    Error(String),
    Screens(ScreensMessage),
}

#[derive(Debug)]
struct FlightCore {
    screen: Screens,
    settings: Arc<RwLock<FlightCoreSettings>>,
}

fn main() -> iced::Result {
    color_eyre::install().expect("couldn't install color_eyre");
    tracing_subscriber::fmt::init();

    iced::application(
        || {
            let settings = Arc::new(RwLock::new(
                tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(FlightCoreSettings::load())
                    .unwrap(),
            ));

            FlightCore {
                screen: Screens::new(Arc::clone(&settings)),
                settings,
            }
        },
        FlightCore::update,
        FlightCore::view,
    )
    .theme(FlightCore::theme)
    .run()
}

impl FlightCore {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Notification(notification) => {
                info!("{notification}");
                Task::none()
            }
            Message::Error(err) => {
                error!("{err}");
                Task::none()
            }
            Message::Screens(message) => self.screen.update(message),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.screen.view()
    }

    fn theme(&self) -> Theme {
        Theme::Ferra
    }
}
