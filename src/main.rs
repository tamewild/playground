use crate::settings::{SettingsMessage, SettingsView};
use iced::widget::{container, row, Container, Row};
use iced::{application, Element, Length, Task};
use crate::chat::{ChatView, ChatViewMsg};

mod openai;
mod settings;
mod chat;

#[derive(Debug)]
enum PlaygroundMessage {
    Chat(ChatViewMsg),
    Settings(SettingsMessage)
}

struct Playground {
    chat_view: ChatView,
    settings_view: SettingsView,
}

impl Playground {
    fn new() -> (Self, Task<PlaygroundMessage>) {
        let (settings_view, task) = SettingsView::new();

        (
            Self { chat_view: ChatView::new(), settings_view },
            task.map(PlaygroundMessage::Settings),
        )
    }

    fn update(&mut self, message: PlaygroundMessage) -> Task<PlaygroundMessage> {
        match message {
            PlaygroundMessage::Chat(msg) => {
                self.chat_view.update(&self.settings_view, msg)
                    .map(PlaygroundMessage::Chat)
            },
            PlaygroundMessage::Settings(msg) => self.settings_view.update(msg),
        }
    }

    fn view(&self) -> Row<PlaygroundMessage> {
        row([
            container(
                Element::from(self.chat_view.view())
                    .map(PlaygroundMessage::Chat)
            )
                .width(Length::FillPortion(3))
                .padding(5.0)
                .into(),
            Element::from(self.settings_view.view()).map(PlaygroundMessage::Settings),
        ])
    }
}

fn main() -> iced::Result {
    application("Playground", Playground::update, Playground::view).run_with(Playground::new)
}
