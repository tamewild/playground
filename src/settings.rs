use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

use iced::widget::{button, column, container, text, text_input, Column, Container, TextInput, slider, row, horizontal_space};
use iced::{Border, Color, Element, Length, Padding, Task, Theme};
use iced::alignment::Horizontal;
use serde::{Deserialize, Serialize};

use crate::PlaygroundMessage;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parsable<T> {
    content: String,
    parsed: Option<T>,
}

impl<T> Parsable<T> {
    pub fn new(object: T) -> Self
        where T: Display
    {
        Self {
            content: object.to_string(),
            parsed: Some(object),
        }
    }

    fn is_valid(&self) -> bool {
        self.parsed.is_some()
    }

    pub fn parsed(&self) -> Option<T>
        where T: Copy
    {
        self.parsed
    }
}

impl<T: FromStr> Parsable<T> {
    fn parse(string: String) -> Self {
        let parsed = string.parse().ok();

        Self {
            content: string,
            parsed,
        }
    }
}

impl<T: Default + Display> Default for Parsable<T> {
    fn default() -> Self {
        let t = T::default();

        Self {
            content: t.to_string(),
            parsed: Some(t),
        }
    }
}

fn parsable_text_input<'a, T: FromStr>(
    placeholder: &'a str,
    parsable: &'a Parsable<T>,
    f: impl 'a + Fn(Parsable<T>) -> SettingsMessage,
) -> TextInput<'a, SettingsMessage> {
    let style_fn = match parsable.parsed {
        None => |theme: &Theme, status| text_input::Style {
            value: theme.palette().danger,
            ..text_input::default(theme, status)
        },
        _ => text_input::default,
    };

    TextInput::new(placeholder, parsable.content.as_str())
        .style(style_fn)
        .on_input(move |changed| f(Parsable::parse(changed)))
}

// rustrover can't resolve the column macro properly, so this is a stopgap
fn pair_in_column<'a>(
    a: impl Into<Element<'a, SettingsMessage>>,
    b: impl Into<Element<'a, SettingsMessage>>,
) -> Column<'a, SettingsMessage> {
    column([a.into(), b.into()])
}

fn default_ui_scale() -> f32 {
    100.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: Parsable<u32>,
    pub temperature: Parsable<f32>,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32
}

impl Default for SerializedSettings {
    fn default() -> Self {
        Self {
            base_url: "".to_string(),
            api_key: "".to_string(),
            model: "".to_string(),
            max_tokens: Parsable::new(1000),
            temperature: Default::default(),
            ui_scale: default_ui_scale(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Load(Box<SerializedSettings>),
    BaseUrlChanged(String),
    ApiKeyChanged(String),
    ModelChanged(String),
    MaxTokensChanged(Parsable<u32>),
    TemperatureChanged(Parsable<f32>),
    UiScaleChanged(f32),
    Save,
    SaveResult(Result<Box<SerializedSettings>, String>),
}

async fn load_existing_settings() -> anyhow::Result<SerializedSettings> {
    let data = tokio::fs::read("settings.json").await?;

    serde_json::from_slice::<SerializedSettings>(data.as_slice()).map_err(Into::into)
}

async fn save_settings(
    serialized_settings: SerializedSettings,
) -> anyhow::Result<SerializedSettings> {
    tokio::fs::write(
        "settings.json",
        serde_json::to_string_pretty(&serialized_settings)?,
    )
    .await
    .map(|_| serialized_settings)
    .map_err(Into::into)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsState {
    /// Latest saved settings (to file) if applicable
    saved_settings: SerializedSettings,
    /// Presented in the UI, may not be saved.
    live_settings: SerializedSettings,
}

impl SettingsState {
    fn valid_parsables(&self) -> bool {
        let settings = &self.live_settings;

        settings.max_tokens.is_valid() && settings.temperature.is_valid()
    }

    fn unsaved_changes(&self) -> bool {
        self.saved_settings != self.live_settings
    }

    pub fn saved(&self) -> &SerializedSettings {
        &self.saved_settings
    }
}

pub enum SettingsView {
    Loading,
    Loaded(SettingsState),
}

impl SettingsView {
    pub fn new() -> (Self, Task<SettingsMessage>) {
        (
            Self::Loading,
            Task::future(async move {
                let settings = load_existing_settings().await.unwrap_or_default();
                SettingsMessage::Load(Box::new(settings))
            }),
        )
    }

    pub fn settings(&self) -> Cow<SettingsState> {
        match self {
            SettingsView::Loading => Cow::Owned(SettingsState::default()),
            SettingsView::Loaded(settings) => Cow::Borrowed(settings),
        }
    }

    fn update_settings<F: FnOnce(&mut SerializedSettings)>(&mut self, f: F) {
        if let SettingsView::Loaded(state) = self {
            f(&mut state.live_settings)
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<PlaygroundMessage> {
        match message {
            SettingsMessage::Load(state) => {
                let state = *state;

                *self = SettingsView::Loaded(SettingsState {
                    saved_settings: state.clone(),
                    live_settings: state,
                });

                Task::none()
            }
            SettingsMessage::BaseUrlChanged(url) => {
                self.update_settings(|settings| settings.base_url = url);

                Task::none()
            }
            SettingsMessage::ApiKeyChanged(api_key) => {
                self.update_settings(|settings| settings.api_key = api_key);

                Task::none()
            }
            SettingsMessage::ModelChanged(model) => {
                self.update_settings(|settings| settings.model = model);

                Task::none()
            }
            SettingsMessage::MaxTokensChanged(max_tokens) => {
                self.update_settings(|settings| settings.max_tokens = max_tokens);

                Task::none()
            }
            SettingsMessage::TemperatureChanged(temperature) => {
                self.update_settings(|settings| settings.temperature = temperature);

                Task::none()
            },
            SettingsMessage::UiScaleChanged(scale) => {
                self.update_settings(|settings| settings.ui_scale = scale);

                Task::none()
            },
            SettingsMessage::Save => {
                let new_settings = self.settings().live_settings.clone();

                Task::future(save_settings(new_settings)).map(|settings| {
                    PlaygroundMessage::Settings(SettingsMessage::SaveResult(
                        settings.map(Box::new).map_err(|err| err.to_string()),
                    ))
                })
            }
            SettingsMessage::SaveResult(res) => {
                // Ignore the error for now
                if let Ok(new_settings) = res {
                    let new_settings = *new_settings;

                    *self = SettingsView::Loaded(SettingsState {
                        saved_settings: new_settings.clone(),
                        live_settings: new_settings,
                    });
                }

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Container<SettingsMessage> {
        container(match self {
            SettingsView::Loading => {
                Element::from(container(text("Loading Settings...")).center(Length::Fill))
            }
            SettingsView::Loaded(settings_state) => {
                let SerializedSettings {
                    base_url,
                    api_key,
                    model,
                    max_tokens,
                    temperature,
                    ui_scale
                } = &settings_state.live_settings;

                column([
                    pair_in_column(
                        "Base URL",
                        text_input("e.g. https://api.openai.com/", base_url)
                            .on_input(SettingsMessage::BaseUrlChanged),
                    )
                    .spacing(5)
                    .into(),
                    pair_in_column(
                        "API Key",
                        text_input("", api_key)
                            .secure(true)
                            .on_input(SettingsMessage::ApiKeyChanged),
                    )
                    .spacing(5)
                    .into(),
                    pair_in_column(
                        "Model",
                        text_input("Model ID e.g. gpt-4o-mini", model)
                            .on_input(SettingsMessage::ModelChanged),
                    )
                    .spacing(4.99) // weird clipping shit with text input
                    .into(),
                    pair_in_column(
                        "Max Tokens",
                        parsable_text_input(
                            "e.g. 1000",
                            max_tokens,
                            SettingsMessage::MaxTokensChanged,
                        ),
                    )
                    .spacing(5)
                    .into(),
                    pair_in_column(
                        "Temperature",
                        parsable_text_input(
                            "e.g. 1.0",
                            temperature,
                            SettingsMessage::TemperatureChanged,
                        ),
                    )
                    .spacing(5)
                    .into(),
                    pair_in_column(
                        row([
                            container(text(format!("UI Scale ({}%)", ui_scale)))
                                .padding(Padding {
                                    top: 5.0,
                                    bottom: 5.0,
                                    right: 0.0,
                                    left: 0.0,
                                })
                                .into(),
                            button("Reset")
                                .style(button::secondary)
                                .on_press(SettingsMessage::UiScaleChanged(100.0))
                                .into()
                        ])
                        .spacing(7),
                        slider(
                            50.0..=150.0,
                            *ui_scale,
                            SettingsMessage::UiScaleChanged
                        )
                    )
                    .spacing(5)
                    .into(),
                    match settings_state.valid_parsables() {
                        true => button(container("Save").center_x(Length::Fill))
                            .on_press_maybe(match settings_state.unsaved_changes() {
                                true => Some(SettingsMessage::Save),
                                false => None,
                            })
                            .into(),
                        false => button(container("Invalid values").center_x(Length::Fill))
                            .style(button::danger)
                            .into(),
                    },
                ])
                .spacing(10)
                .into()
            }
        })
        .style(|theme| {
            container::rounded_box(theme)
                .border(Border::default())
        })
        .padding(5.0)
        .width(Length::Fill)
        .height(Length::Fill)
    }
}

#[cfg(test)]
mod tests {
    use crate::PlaygroundMessage;
    use crate::settings::{SerializedSettings, SettingsMessage, SettingsState};

    #[test]
    fn size() {
        dbg!(size_of::<SerializedSettings>());
        dbg!(size_of::<SettingsState>());
        dbg!(size_of::<SettingsMessage>());
        dbg!(size_of::<PlaygroundMessage>());
    }
}