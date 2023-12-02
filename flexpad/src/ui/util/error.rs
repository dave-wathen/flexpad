use crate::ui::{
    style::{self, DialogStyle},
    util::{
        button_bar, dialog::Dialog, dialog_button, dialog_title, handle_ok_and_cancel_keys, SPACE_S,
    },
    Action,
};
use iced::{
    subscription,
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
        .style(DialogStyle::Error)
        .max_width(400.0)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Acknowledge => Action::CloseDialog,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, _status| {
            handle_ok_and_cancel_keys(&event, Message::Acknowledge, Message::Acknowledge)
        })
    }
}