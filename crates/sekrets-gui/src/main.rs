mod screen;
mod style;

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

    fn theme(&self) -> iced::Theme {
        style::theme()
    }

    fn subscription(&self) -> iced::Subscription<screen::Message> {
        // Every screen that holds — or has in flight — a decrypted vault must tick, or it
        // never auto-locks. `MigrationPrompt` is one a user can sit on indefinitely.
        match &self.screen {
            Some(screen::Screen::Unlocked { .. })
            | Some(screen::Screen::MigrationPrompt { .. })
            | Some(screen::Screen::Migrating { .. }) => {
                iced::time::every(std::time::Duration::from_secs(5)).map(|_| screen::Message::Tick)
            }
            _ => iced::Subscription::none(),
        }
    }
}

fn main() -> iced::Result {
    iced::application(SekretsApp::title, SekretsApp::update, SekretsApp::view)
        .subscription(SekretsApp::subscription)
        .theme(SekretsApp::theme)
        .window_size(iced::Size::new(720.0, 800.0))
        .centered()
        .run_with(SekretsApp::new)
}
