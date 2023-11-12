use crate::ui::{
    button_bar,
    dialog::Dialog,
    dialog_button, dialog_title, handle_ok_and_cancel_keys,
    style::{self, DialogStyle},
    SPACE_S,
};
use iced::{
    subscription,
    widget::{column, text},
    Subscription,
};
use rust_i18n::t;

use super::WorkpadMessage;

#[derive(Debug)]
pub struct ErrorUi {
    message: String,
}

// TODO Focus management
impl ErrorUi {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn view(&self) -> iced::Element<'_, WorkpadMessage> {
        let ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Error)
            .on_press(WorkpadMessage::ModalCancel);

        let body = column![text(&self.message), button_bar().push(ok)].spacing(SPACE_S);

        Dialog::new(
            dialog_title(t!("Error.Title"), style::DialogStyle::Error),
            body,
        )
        .style(DialogStyle::Error)
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<WorkpadMessage> {
        subscription::events_with(|event, _status| {
            handle_ok_and_cancel_keys(
                &event,
                WorkpadMessage::ModalCancel,
                WorkpadMessage::ModalCancel,
            )
        })
    }
}
