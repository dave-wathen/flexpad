use crate::ui::{action::ActionSet, SPACE_M};
use iced::{
    theme,
    widget::{column, text},
    Subscription,
};
use iced_aw::{Card, CardStyles};
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
        Card::new(
            text(t!("Error.Title")),
            column![text(&self.message)].spacing(SPACE_M),
        )
        .foot(
            ActionSet::ok()
                .ok_button_style(theme::Button::Destructive)
                .to_element()
                .map(|_| WorkpadMessage::ModalCancel),
        )
        .style(CardStyles::Danger)
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<WorkpadMessage> {
        ActionSet::ok()
            .to_subscription()
            .map(|_| WorkpadMessage::ModalCancel)
    }
}
