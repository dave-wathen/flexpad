use iced::{
    alignment::Horizontal,
    keyboard, subscription, theme,
    widget::{button, horizontal_space, row, text},
    Event, Length, Subscription,
};

use super::{DIALOG_BUTTON_WIDTH, SPACE_S};

#[derive(Debug, Clone, Copy)]
pub enum OkCancel {
    Ok,
    Cancel,
}

impl OkCancel {
    pub fn buttons_view<'a>() -> iced::Element<'a, OkCancel> {
        row![
            horizontal_space(Length::Fill),
            row![
                button(text("Cancel").horizontal_alignment(Horizontal::Center))
                    .width(DIALOG_BUTTON_WIDTH)
                    .style(theme::Button::Secondary)
                    .on_press(OkCancel::Cancel),
                button(text("Ok").horizontal_alignment(Horizontal::Center))
                    .width(DIALOG_BUTTON_WIDTH)
                    .on_press(OkCancel::Ok),
            ]
            .spacing(SPACE_S)
        ]
        .width(Length::Fill)
        .into()
    }

    pub fn subscription() -> Subscription<OkCancel> {
        subscription::events_with(|event, _status| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) => match key_code {
                keyboard::KeyCode::Escape if modifiers.is_empty() => Some(OkCancel::Cancel),
                keyboard::KeyCode::Enter if modifiers.is_empty() => Some(OkCancel::Ok),
                _ => None,
            },
            _ => None,
        })
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, OkCancel::Ok)
    }

    pub fn is_cancel(&self) -> bool {
        matches!(self, OkCancel::Cancel)
    }
}
