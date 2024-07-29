use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct SerializedSettings {
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: Parsable<u32>,
    temperature: Parsable<f32>
}

pub enum SettingsMessage {
    BaseUrlChanged(String),
    ApiKeyChanged(String),
    ModelChanged(String),
    MaxTokensChanged(Parsable<u32>),
    TemperatureChanged(Parsable<f32>)
}

async fn serialized_settings() -> std::io::Result<SerializedSettings> {
    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    serialized_settings: SerializedSettings,
    changed: bool
}

impl Settings {

}