use std::borrow::Cow;
use std::str::FromStr;
use iced::futures::TryFutureExt;
use iced::{Length, Task};
use iced::widget::{Container, container, text, text_input};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Parsable<T> {
    Valid(T),
    Invalid(String)
}

impl<T: FromStr> Parsable<T> {
    fn parse(string: String) -> Self {
        string.parse::<T>()
            .map(Self::Valid)
            .unwrap_or(Parsable::Invalid(string))
    }
}

impl<T: Default> Default for Parsable<T> {
    fn default() -> Self {
        Self::Valid(T::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct SerializedSettings {
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: Parsable<u32>,
    temperature: Parsable<f32>
}

#[derive(Debug)]
pub enum SettingsMessage {
    Load(SettingsState),
    BaseUrlChanged(String),
    ApiKeyChanged(String),
    ModelChanged(String),
    MaxTokensChanged(Parsable<u32>),
    TemperatureChanged(Parsable<f32>)
}

async fn load_existing_settings() -> anyhow::Result<SerializedSettings> {
    let data = tokio::fs::read("settings.json").await?;

    serde_json::from_slice::<SerializedSettings>(data.as_slice())
        .map_err(Into::into)
}

async fn save_settings(serialized_settings: &SerializedSettings) -> anyhow::Result<()> {
    tokio::fs::write(
        "settings.json",
        serde_json::to_string_pretty(serialized_settings)?
    ).await.map_err(Into::into)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsState {
    /// Latest saved settings (to file) if applicable
    saved_settings: SerializedSettings,
    /// Presented in the UI, may not be saved.
    live_settings: SerializedSettings
}

pub enum SettingsView {
    Loading,
    Loaded(SettingsState)
}

impl SettingsView {
    pub fn new() -> (Self, Task<SettingsMessage>) {
        (Self::Loading, Task::future(async move {
            let settings = load_existing_settings().await.unwrap_or_default();
            SettingsMessage::Load(SettingsState {
                saved_settings: settings.clone(),
                live_settings: settings,
            })
        }))
    }

    pub fn settings(&self) -> Cow<SettingsState> {
        match self {
            SettingsView::Loading => Cow::Owned(SettingsState::default()),
            SettingsView::Loaded(settings) => Cow::Borrowed(settings)
        }
    }

    pub fn update(&mut self, message: SettingsMessage) {
        match message {
            SettingsMessage::Load(state) => {
                *self = SettingsView::Loaded(state);
            }
            SettingsMessage::BaseUrlChanged(_) => {}
            SettingsMessage::ApiKeyChanged(_) => {}
            SettingsMessage::ModelChanged(_) => {}
            SettingsMessage::MaxTokensChanged(_) => {}
            SettingsMessage::TemperatureChanged(_) => {}
        }
    }

    pub fn view(&self) -> Container<SettingsMessage> {
        container(
            container(text("Loading Settings...")).center(Length::Fill)
        )
            .style(container::rounded_box)
            .width(Length::Fill)
            .height(Length::Fill)
    }
}