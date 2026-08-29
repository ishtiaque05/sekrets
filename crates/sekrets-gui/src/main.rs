mod screen;

use screen::SekretsApp;

impl SekretsApp {
    fn new() -> (Self, iced::Task<screen::Message>) {
        let locate = iced::Task::perform(
            async { sekrets_core::Vault::locate() },
            screen::Message::Located,
        );
        (
            SekretsApp {
                screen: Some(screen::Screen::Locating),
            },
            locate,
        )
    }

    fn title(&self) -> String {
        "Sekrets".to_string()
    }

    fn subscription(&self) -> iced::Subscription<screen::Message> {
        match &self.screen {
            Some(screen::Screen::Unlocked { .. }) => {
                iced::time::every(std::time::Duration::from_secs(5)).map(|_| screen::Message::Tick)
            }
            _ => iced::Subscription::none(),
        }
    }
}

fn main() -> iced::Result {
    iced::application(SekretsApp::title, SekretsApp::update, SekretsApp::view)
        .subscription(SekretsApp::subscription)
        .run_with(SekretsApp::new)
}
