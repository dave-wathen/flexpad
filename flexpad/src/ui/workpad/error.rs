use crate::ui::{
    action::{ActionSet, ActionStyle},
    SPACE_M,
};
use iced::{
    theme,
    widget::{column, text},
    Subscription,
};
use iced_aw::{style, Card, CardStyles};
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
                .ok_button_style(ActionStyle::Error)
                .to_element()
                .map(|_| WorkpadMessage::ModalCancel),
        )
        .style(CardStyles::Custom(Box::new(ErrorStyle)))
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<WorkpadMessage> {
        ActionSet::ok()
            .to_subscription()
            .map(|_| WorkpadMessage::ModalCancel)
    }
}

struct ErrorStyle;

impl style::card::StyleSheet for ErrorStyle {
    type Style = theme::Theme;

    fn active(&self, theme: &Self::Style) -> iced_aw::card::Appearance {
        let palette = theme.extended_palette();
        let pair = palette.danger.base;

        iced_aw::card::Appearance {
            border_color: pair.color,
            head_background: pair.color.into(),
            head_text_color: pair.text,
            close_color: pair.text,
            background: palette.background.base.color.into(),
            body_text_color: theme.palette().text,
            foot_text_color: theme.palette().text,
            ..iced_aw::card::Appearance::default()
        }
    }
}
