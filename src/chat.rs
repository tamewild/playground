use std::sync::Arc;

use iced::{border, Color, Length, task, Task};
use iced::widget::{button, checkbox, column, Column, Container, container, horizontal_space, pick_list, row, scrollable, Scrollable, text_editor};
use iced::widget::text_editor::{Action, Edit};

use crate::openai::{CompletionRequest, Message, Role};
use crate::openai;
use crate::settings::SettingsView;

#[derive(Debug, Clone)]
pub enum ChatViewMsg {
    ChangeRole {
        index: usize,
        role: Role
    },
    EditText {
        index: usize,
        action: Action
    },
    AddMessage,
    DeleteMessage {
        index: usize
    },
    Run,
    Stop,
    Completion {
        delta: Result<String, String>
    },
    StickToBottom(bool)
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
            {
                let mut editor = text_editor(&message.content)
                    .placeholder(match message.role {
                        Role::System => "Set a system prompt...",
                        Role::User => "Enter your prompt...",
                        Role::Assistant => "Enter the assistant's response..."
                    });

                if not_inferencing {
                    editor = editor.on_action(move |action| {
                        ChatViewMsg::EditText {
                            index,
                            action
                        }
                    })
                }

                editor.into()
            }
        ])
            .spacing(5.0)
    )
        .style(container::rounded_box)
        .padding(5.0)
}


enum InferenceStatus {
    Idle,
    Inferencing {
        #[allow(dead_code)]
        abort_handle: task::Handle
    }
}

pub struct ChatView {
    messages: Vec<UiChatMsg>,
    inference_status: InferenceStatus,
    stick_to_bottom: bool,
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
            inference_status: InferenceStatus::Idle,
            stick_to_bottom: false,
        }
    }

    pub fn update(&mut self, settings_view: &SettingsView, msg: ChatViewMsg) -> Task<ChatViewMsg> {
        match msg {
            ChatViewMsg::ChangeRole { index, role } => {
                if matches!(self.inference_status, InferenceStatus::Idle) {
                    self.messages[index].role = role;
                }

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
                            self.inference_status = InferenceStatus::Idle;

                            format!("\n\nRan into an error:\n{err}")
                        }))
                    )))
                }

                Task::none()
            }
            ChatViewMsg::StickToBottom(value) => {
                self.stick_to_bottom = value;

                Task::none()
            }
        }
    }

    fn message_list(&self, not_inferencing: bool) -> Scrollable<ChatViewMsg> {
        let mut scrollable = scrollable(
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
                            button("+ Add Message")
                                .on_press_maybe(
                                    not_inferencing
                                        .then_some(ChatViewMsg::AddMessage)
                                )
                                .style(button::secondary)
                        )
                            .center_x(Length::Fill)
                            .into()
                    ))
            )
                .spacing(10)
        )
            .spacing(3);

        if self.stick_to_bottom {
            scrollable = scrollable.anchor_bottom();
        }

        scrollable
    }

    pub fn view(&self) -> Column<ChatViewMsg> {
        let not_inferencing = matches!(self.inference_status, InferenceStatus::Idle);

        column([
            container(self.message_list(not_inferencing))
                .style(|_| {
                    container::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: border::rounded(2)
                            .width(1.0)
                            .color(Color::from_rgba8(107, 107, 107, 0.5)),
                        ..Default::default()
                    }
                })
                .height(Length::Fill)
                .padding(5)
                .into(),
            container(
                row([
                    match self.inference_status {
                        InferenceStatus::Idle => {
                            button(
                                container("Run")
                                    .center_x(Length::Fill)
                            )
                                .on_press(ChatViewMsg::Run)
                        }
                        InferenceStatus::Inferencing { .. } => {
                            button(
                                container("Stop")
                                    .center_x(Length::Fill)
                            )
                                .style(button::danger)
                                .on_press(ChatViewMsg::Stop)
                        }
                    }
                        .into(),
                    button(
                        checkbox("Stick to Bottom", self.stick_to_bottom)
                            .on_toggle(ChatViewMsg::StickToBottom)
                    )
                        .style(|_, _| button::Style {
                            text_color: Color::WHITE,
                            ..Default::default()
                        })
                        .into(),
                    horizontal_space()
                        .width(Length::FillPortion(4))
                        .into()
                ])
            )
                .width(Length::Fill)
                .height(Length::Shrink)
                .into()
        ])
            .spacing(5.0)
    }
}