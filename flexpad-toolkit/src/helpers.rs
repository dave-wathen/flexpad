use crate::action::Action;
use iced::{Element, Pixels};

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
