use iced::Length;
use iced::widget::{button, column, Column, Container, container, horizontal_space, pick_list, row, text, text_editor};

use crate::openai::{Message, Role};

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

pub struct ChatView {
    messages: Vec<UiChatMsg>,
    inferencing: bool
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
            inferencing: false,
        }
    }

    pub fn update(&mut self, msg: ChatViewMsg) {
        match msg {
            ChatViewMsg::ChangeRole { index, role } => {
                self.messages[index].role = role;
            }
            ChatViewMsg::EditText { index, action } => {
                self.messages[index].content.perform(action)
            }
            ChatViewMsg::AddMessage => {
                self.messages.push(UiChatMsg {
                    role: Role::User,
                    content: text_editor::Content::new(),
                })
            }
            ChatViewMsg::DeleteMessage { index } => {
                self.messages.remove(index);
            }
        }
    }

    pub fn view(&self) -> Column<ChatViewMsg> {
        let not_inferencing = !self.inferencing;

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
                                button("Run"),
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