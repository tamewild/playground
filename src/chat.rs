use iced::futures::AsyncReadExt;
use iced::widget::{button, column, Column, Container, container, pick_list, row, Row, text_editor};

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

fn message_widget((index, message): (usize, &UiChatMsg)) -> Container<ChatViewMsg> {
    container(
        column([
            pick_list(UiChatMsg::ROLES, Some(message.role), move |role| {
                ChatViewMsg::ChangeRole {
                    index,
                    role
                }
            }).into(),
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
    messages: Vec<UiChatMsg>
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
        }
    }

    pub fn view(&self) -> Column<ChatViewMsg> {
        column(
            self.messages
                .iter()
                .enumerate()
                .map(message_widget)
                .map(Into::into)
                .chain(std::iter::once(
                    button("+ Add Message")
                        .into()
                ))
        )
            .spacing(10)
    }
}