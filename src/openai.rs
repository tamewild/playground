use anyhow::anyhow;
use iced::futures::{Stream, StreamExt, TryStreamExt};
use reqwest::header::AUTHORIZATION;
use reqwest_eventsource::{Event, RequestBuilderExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display, Formatter};
use std::future;
use std::sync::OnceLock;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Role as Debug>::fmt(self, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    messages: Vec<Message>,
    model: String,
    max_tokens: u32,
    stream: bool,
    temperature: f32,
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>, model: String, max_tokens: u32, temperature: f32) -> Self {
        Self {
            messages,
            model,
            max_tokens,
            stream: true,
            temperature,
        }
    }
}

/// Returns a completions stream with the completion delta as each item
pub fn completions(
    base_url: &str,
    api_key: &str,
    request: CompletionRequest,
) -> impl Stream<Item = anyhow::Result<String>> {
    const COMPLETIONS_PATH: &str = "v1/chat/completions";

    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    let client = CLIENT.get_or_init(reqwest::Client::new);

    let url = match base_url.chars().last() {
        Some('/') => format!("{base_url}{COMPLETIONS_PATH}"),
        _ => format!("{base_url}/{COMPLETIONS_PATH}"),
    };

    client
        .post(url)
        .json(&request)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .eventsource()
        .unwrap() // Impossible
        .take_while(|res| {
            future::ready(!matches!(res, Err(reqwest_eventsource::Error::StreamEnded)))
        })
        .map_err(Into::into)
        .try_filter_map(|event| async move {
            Ok(match event {
                Event::Message(event) => {
                    if event.data.as_str() == "[DONE]" {
                        return Ok(None);
                    }

                    let value = serde_json::from_str::<Value>(event.data.as_str())?;
                    Some(
                        value
                            .pointer("/choices/0/delta/content")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .ok_or_else(|| anyhow!("Delta not found within:\n{value:#}"))?,
                    )
                }
                _ => None,
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::openai::{CompletionRequest, Message, Role};
    use iced::futures::TryStreamExt;

    #[tokio::test]
    async fn together() {
        let api_key = std::env::var("TOGETHER_API_KEY").unwrap();

        println!("Using api key: {api_key}");

        let req = CompletionRequest::new(
            vec![Message {
                content: "hi".to_string(),
                role: Role::User,
            }],
            "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo".to_string(),
            1000,
            0.0,
        );

        super::completions("https://api.together.xyz/", api_key.as_str(), req)
            .try_for_each(|delta| async move {
                println!("{delta}");
                Ok(())
            })
            .await
            .unwrap();
    }
}
