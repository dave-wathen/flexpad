use crate::{action::Action, action_button::ActionButton, shortcut::Shortcut};
use iced::{
    alignment,
    widget::{button, text, Text},
    Element, Font, Length, Pixels,
};

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../resources/flexpad-icons.ttf");
pub const ICON_FONT: Font = Font::with_name("flexpad-icons");

pub const SPACE_S: f32 = 5.0;
pub const SPACE_M: f32 = SPACE_S * 2.0;
pub const SPACE_L: f32 = SPACE_S * 4.0;
pub const SPACE_XL: f32 = SPACE_S * 8.0;

pub const TEXT_SIZE_TOOLTIP: Pixels = Pixels(12.0);

pub type Tooltip<'a, Message> = iced::widget::Tooltip<'a, Message, iced::Renderer>;
pub type TooltipPosition = iced::widget::tooltip::Position;

/// Create a tooltip using the content of the given [`Action`]
pub fn tooltip<'a, Message>(
    action: &Action,
    content: impl Into<Element<'a, Message, iced::Renderer>>,
    position: TooltipPosition,
) -> Tooltip<'a, Message> {
    let label = match action.shortcut {
        Some(key) => format!("{}  {}", action.name, key),
        None => action.name.clone(),
    };

    iced::widget::tooltip(content, label, position)
        .size(TEXT_SIZE_TOOLTIP)
        .style(iced::theme::Container::Box)
}

/// Create an icon for a given codepoint in the flexpad-icons fonts.
pub fn icon<'a>(codepoint: char, size: impl Into<Pixels>) -> Text<'a, iced::Renderer> {
    text(codepoint)
        .font(ICON_FONT)
        .size(size.into())
        .line_height(1.0)
        .shaping(text::Shaping::Advanced)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center)
        .height(Length::Fill)
        .vertical_alignment(alignment::Vertical::Center)
}

/// A button representing an [`Action`]
pub fn action_button<Message, Renderer>(
    action: impl Into<Action>,
) -> ActionButton<Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: button::StyleSheet,
{
    ActionButton::new(action)
}

/// A container that implements an [`Action`] shortcut for some [`Element`]
pub fn shortcut<'a, Message, Renderer>(
    action: impl Into<Action>,
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Shortcut<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: button::StyleSheet,
{
    Shortcut::new(action, content)
}
