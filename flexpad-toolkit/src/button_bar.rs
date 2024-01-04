use crate::prelude::SPACE_M;
use iced::{
    widget::{horizontal_space, row, Button, Row},
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
{
    /// Create an empty [`ButtonBar`]
    pub fn new() -> Self
    where
        Message: 'a,
        Renderer: 'a + iced::advanced::Renderer,
    {
        Self {
            row: row![horizontal_space(Length::Fill)].spacing(SPACE_M),
        }
    }

    /// Add a button to the [`ButtonBar`]
    pub fn push(self, button: Button<'a, Message, Renderer>) -> Self {
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
