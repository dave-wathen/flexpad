use crate::{action_button::ActionButton, prelude::SPACE_M};
use iced::{
    widget::{horizontal_space, row, Row},
    Element, Length,
};

pub struct ButtonBar<'a, Message, Renderer> {
    row: Row<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> ButtonBar<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: iced::widget::button::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: iced::widget::text::StyleSheet,
{
    /// Create an empty [`ButtonBar`]
    pub fn new() -> Self {
        Self {
            row: row![horizontal_space(Length::Fill)].spacing(SPACE_M),
        }
    }

    /// Add an [`ActionButton`] to the [`ButtonBar`]
    pub fn push(self, button: ActionButton<Message, Renderer>) -> Self {
        let Self { row } = self;
        Self {
            row: row.push(button),
        }
    }
}

impl<'a, Message, Renderer> Default for ButtonBar<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: iced::widget::button::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: iced::widget::text::StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Renderer> From<ButtonBar<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(value: ButtonBar<'a, Message, Renderer>) -> Self {
        value.row.into()
    }
}
