mod screen;

use iced::widget::text;
use iced::{Element, Task};
use screen::{Message, Screen, SekretsApp};

impl SekretsApp {
    fn new() -> (Self, Task<Message>) {
        let locate = Task::perform(async { sekrets_core::Vault::locate() }, Message::Located);
        (
            SekretsApp {
                screen: Some(Screen::Locating),
            },
            locate,
        )
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Some(Screen::Locating) => text("Looking for your sekrets file...").into(),
            Some(Screen::NoVaultFound { path }) => {
                text(format!("No sekrets file found at {}", path.display())).into()
            }
            Some(Screen::Locked { path }) => {
                text(format!("Found sekrets file at {}", path.display())).into()
            }
            None => text("").into(),
        }
    }

    fn title(&self) -> String {
        "Sekrets".to_string()
    }
}

fn main() -> iced::Result {
    iced::application(SekretsApp::title, SekretsApp::update, SekretsApp::view)
        .run_with(SekretsApp::new)
}
