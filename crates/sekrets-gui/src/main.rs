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
}

fn main() -> iced::Result {
    iced::application(SekretsApp::title, SekretsApp::update, SekretsApp::view)
        .run_with(SekretsApp::new)
}
