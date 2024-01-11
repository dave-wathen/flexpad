use crate::{action::Action, action_button::ActionButton, shortcut::Shortcut, style};
use iced::{
    alignment,
    widget::{self, button, column, container, text, vertical_space, Text},
    Element, Font, Length, Pixels,
};

pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../resources/flexpad-icons.ttf");
pub const ICON_FONT: Font = Font::with_name("flexpad-icons");

pub const ICON_BUTTON_SIZE: Pixels = Pixels(48.0);

pub const ICON_FX: char = '\u{E81A}';
pub const ICON_OPEN_DOWN: char = '\u{E806}';

pub const SPACE_S: f32 = 5.0;
pub const SPACE_M: f32 = SPACE_S * 2.0;
pub const SPACE_L: f32 = SPACE_S * 4.0;
pub const SPACE_XL: f32 = SPACE_S * 8.0;

pub const TEXT_SIZE_APP_TITLE: Pixels = Pixels(20.0);
pub const TEXT_SIZE_DIALOG_TITLE: Pixels = Pixels(16.0);
pub const TEXT_SIZE_ERROR: Pixels = Pixels(14.0);
pub const TEXT_SIZE_INPUT: Pixels = Pixels(16.0);
pub const TEXT_SIZE_LABEL: Pixels = Pixels(12.0);
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

// Create a title for a dialog box
pub fn dialog_title<'a, Message>(
    title: impl ToString,
    style: style::DialogStyle,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(title).size(TEXT_SIZE_DIALOG_TITLE).style(style)).into()
}

pub fn label<'a, Message>(label: impl ToString) -> Element<'a, Message> {
    iced::widget::text(label)
        .size(TEXT_SIZE_LABEL)
        .style(style::TextStyle::Label)
        .into()
}

pub fn text_input<'a, Message, F>(
    label: impl ToString,
    placeholder: impl ToString,
    value: &str,
    on_input: F,
    error: Option<&String>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    F: 'a + Fn(String) -> Message,
{
    let below: Element<'a, Message> = match error {
        Some(msg) => container(
            text(msg)
                .size(TEXT_SIZE_ERROR)
                .style(style::TextStyle::Error),
        )
        .height(SPACE_L)
        .into(),
        None => vertical_space(SPACE_L).into(),
    };

    let input_style = match error {
        Some(_) => style::TextInputStyle::Error,
        None => style::TextInputStyle::Default,
    };

    let icon = match error {
        Some(_) => widget::text_input::Icon {
            font: Font::default(),
            code_point: '\u{2757}',
            size: Some(TEXT_SIZE_INPUT),
            spacing: SPACE_M,
            side: widget::text_input::Side::Right,
        },
        None => widget::text_input::Icon {
            font: Font::default(),
            code_point: '\u{2713}',
            size: Some(TEXT_SIZE_INPUT),
            spacing: SPACE_M,
            side: widget::text_input::Side::Right,
        },
    };

    column![
        self::label(label),
        widget::vertical_space(SPACE_S),
        iced::widget::text_input(&placeholder.to_string(), value)
            .size(TEXT_SIZE_INPUT)
            .icon(icon)
            .style(input_style)
            .on_input(on_input),
        below
    ]
    .into()
}
