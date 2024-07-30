use std::sync::Arc;
use iced::{Length, task, Task};
use iced::widget::{button, column, Column, Container, container, horizontal_space, pick_list, row, text, text_editor};
use iced::widget::text_editor::{Action, Edit};
use crate::openai::{CompletionRequest, Message, Role};
use crate::{openai, Playground, PlaygroundMessage};
use crate::settings::SettingsView;

#[derive(Debug, Clone)]
pub enum ChatViewMsg {
    ChangeRole {
        index: usize,
        role: Role
    },
    EditText {
        index: usize,
        action: text_editor::Action
    },
    AddMessage,
    DeleteMessage {
        index: usize
    },
    Run,
    Stop,
    Completion {
        delta: Result<String, String>
    }
}

struct UiChatMsg {
    role: Role,
    content: text_editor::Content
}

impl UiChatMsg {
    const ROLES: &'static [Role] = &[
        Role::System,
        Role::User,
        Role::Assistant
    ];
}

fn message_widget((index, message): (usize, &UiChatMsg), not_inferencing: bool) -> Container<ChatViewMsg> {
    container(
        column([
            row([
                pick_list(UiChatMsg::ROLES, Some(message.role), move |role| {
                    ChatViewMsg::ChangeRole {
                        index,
                        role
                    }
                })
                    .into(),
                horizontal_space().into(),
                button("Delete")
                    .style(button::danger)
                    .on_press_maybe(
                        not_inferencing
                            .then_some(ChatViewMsg::DeleteMessage {
                                index
                            })
                    )
                    .into()
            ])
                .into(),
            text_editor(&message.content)
                .placeholder(match message.role {
                    Role::System => "Set a system prompt...",
                    Role::User => "Enter your prompt...",
                    Role::Assistant => "Enter the assistant's response..."
                })
                .on_action(move |action| {
                    ChatViewMsg::EditText {
                        index,
                        action
                    }
                })
                .into()
        ])
            .spacing(5.0)
    )
        .style(container::rounded_box)
        .padding(5.0)
}


enum InferenceStatus {
    Idle,
    Inferencing {
        abort_handle: task::Handle
    }
}

pub struct ChatView {
    messages: Vec<UiChatMsg>,
    inference_status: InferenceStatus
}

impl ChatView {
    pub fn new() -> Self {
        Self {
            messages: vec![
                UiChatMsg {
                    role: Role::User,
                    content: text_editor::Content::new(),
                }
            ],
            inference_status: InferenceStatus::Idle
        }
    }

    pub fn update(&mut self, settings_view: &SettingsView, msg: ChatViewMsg) -> Task<ChatViewMsg> {
        match msg {
            ChatViewMsg::ChangeRole { index, role } => {
                self.messages[index].role = role;

                Task::none()
            }
            ChatViewMsg::EditText { index, action } => {
                self.messages[index].content.perform(action);

                Task::none()
            }
            ChatViewMsg::AddMessage => {
                self.messages.push(UiChatMsg {
                    role: Role::User,
                    content: text_editor::Content::new(),
                });

                Task::none()
            }
            ChatViewMsg::DeleteMessage { index } => {
                self.messages.remove(index);

                Task::none()
            }
            ChatViewMsg::Run => {
                let settings = settings_view.settings();

                let saved_settings = settings.saved();

                let req = CompletionRequest::new(
                    self.messages.iter()
                        .map(|ui_msg| Message {
                            content: ui_msg.content.text(),
                            role: ui_msg.role,
                        })
                        .collect(),
                    saved_settings.model.clone(),
                    saved_settings.max_tokens.parsed().unwrap_or_default(),
                    saved_settings.temperature.parsed().unwrap_or_default()
                );

                let (task, abort_handle) = Task::stream(
                    openai::completions(
                        saved_settings.base_url.as_str(),
                        saved_settings.api_key.as_str(),
                        req
                    )
                )
                    .map(|res| {
                        ChatViewMsg::Completion {
                            delta: res.map_err(|err| err.to_string())
                        }
                    })
                    .chain(Task::done(ChatViewMsg::Stop))
                    .abortable();

                self.inference_status = InferenceStatus::Inferencing {
                    abort_handle: abort_handle.abort_on_drop(),
                };

                let is_last_msg_assistant =
                    self.messages.last().is_some_and(|msg| msg.role == Role::Assistant);

                if !is_last_msg_assistant {
                    self.messages.push(UiChatMsg {
                        role: Role::Assistant,
                        content: text_editor::Content::new(),
                    })
                }

                task
            }
            ChatViewMsg::Stop => {
                self.inference_status = InferenceStatus::Idle;

                Task::none()
            }
            ChatViewMsg::Completion { delta } => {
                if let Some(msg) = self.messages.last_mut() {
                    msg.content.perform(Action::Edit(Edit::Paste(
                        Arc::new(delta.unwrap_or_else(|err| {
                            format!("\n\nRan into an error:\n{err}")
                        }))
                    )))
                }

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Column<ChatViewMsg> {
        let not_inferencing = matches!(self.inference_status, InferenceStatus::Idle);

        column(
            self.messages
                .iter()
                .enumerate()
                .map(|pair| {
                    message_widget(pair, not_inferencing)
                })
                .map(Into::into)
                .chain(std::iter::once(
                    container(
                        row(
                            [
                                match not_inferencing {
                                    true => button("Run").on_press(ChatViewMsg::Run),
                                    false => button("Stop")
                                        .style(button::danger)
                                        .on_press(ChatViewMsg::Stop)
                                },
                                button("+ Add Message")
                                    .on_press_maybe(
                                        not_inferencing
                                            .then_some(ChatViewMsg::AddMessage)
                                    )
                                    .style(button::secondary)
                            ]
                                .map(Into::into)
                        )
                            .spacing(5)
                    )
                        .center_x(Length::Fill)
                        .into()
                ))
        )
            .spacing(10)
    }
}