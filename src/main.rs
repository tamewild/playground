use iced::{application, Element, Length, Task};
use iced::widget::{container, Container, row, Row};
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

    fn update(&mut self, message: PlaygroundMessage) -> Task<PlaygroundMessage> {
        match message {
            PlaygroundMessage::Settings(msg) => {
                self.settings_view.update(msg)
            }
        }
    }

    fn view(&self) -> Row<PlaygroundMessage> {
        row([
            container(chats_placeholder())
                .width(Length::FillPortion(3))
                .into(),
            Element::from(self.settings_view.view())
                .map(PlaygroundMessage::Settings)
        ])
    }
}

fn chats_placeholder() -> Container<'static, PlaygroundMessage> {
    container("Chats would be here")
        .center(Length::Fill)
}

fn main() -> iced::Result {
    application("Playground", Playground::update, Playground::view)
        .run_with(Playground::new)
}
