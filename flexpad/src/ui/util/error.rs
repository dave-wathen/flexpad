use crate::ui::util::{button_bar, dialog_button, dialog_title, handle_ok_and_cancel_keys};
use flexpad_toolkit::{dialog::Dialog, prelude::*};
use iced::{
    event,
    widget::{column, text},
    Subscription,
};
use rust_i18n::t;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Acknowledge,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error::Message::")?;
        match self {
            Self::Acknowledge => write!(f, "Acknowledge"),
        }
    }
}

pub enum Event {
    Acknowledged,
}

#[derive(Debug)]
pub struct ErrorUi {
    message: String,
}

// TODO Focus management
impl ErrorUi {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Error)
            .on_press(Message::Acknowledge);

        let body = column![text(&self.message), button_bar().push(ok)].spacing(SPACE_S);

        Dialog::new(
            dialog_title(t!("Error.Title"), style::DialogStyle::Error),
            body,
        )
        .style(style::DialogStyle::Error)
        .max_width(400.0)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Acknowledge => Event::Acknowledged,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status| {
            handle_ok_and_cancel_keys(&event, Message::Acknowledge, Message::Acknowledge)
        })
    }
}
