use iced::{application, Element, Task};
use crate::settings::{SettingsMessage, SettingsView};

mod openai;
mod settings;

#[derive(Debug)]
enum PlaygroundMessage {
    Settings(SettingsMessage)
}

struct Playground {
    settings_view: SettingsView
}

impl Playground {
    fn new() -> (Self, Task<PlaygroundMessage>) {
        let (settings_view, task) = SettingsView::new();

        (
            Self {
                settings_view,
            },
            task.map(PlaygroundMessage::Settings)
        )
    }

    fn update(&mut self, message: PlaygroundMessage) {

    }

    fn view(&self) -> Element<PlaygroundMessage> {
        Element::from(self.settings_view.view())
            .map(PlaygroundMessage::Settings)
    }
}

fn main() -> iced::Result {
    application("Playground", Playground::update, Playground::view)
        .run_with(Playground::new)
}
