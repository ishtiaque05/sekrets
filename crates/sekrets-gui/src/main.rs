use iced::widget::text;
use iced::Element;

#[derive(Default)]
struct SekretsApp;

#[derive(Debug, Clone)]
enum Message {}

impl SekretsApp {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        text("Sekrets").size(24).into()
    }
}

fn main() -> iced::Result {
    iced::run("Sekrets", SekretsApp::update, SekretsApp::view)
}
