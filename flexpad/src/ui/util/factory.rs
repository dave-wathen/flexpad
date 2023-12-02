use iced::{
    alignment, keyboard, theme,
    widget::{
        self, button, column, container, horizontal_space, row, text, vertical_space, Button, Row,
    },
    Element, Event, Font, Length,
};

use crate::ui::style;

pub const SPACE_S: f32 = 5.0;
pub const SPACE_M: f32 = SPACE_S * 2.0;
pub const SPACE_L: f32 = SPACE_S * 4.0;
// const SPACE_XL: u16 = SPACE_S * 8;

pub const TEXT_SIZE_DIALOG_TITLE: f32 = 16.0;
pub const TEXT_SIZE_LABEL: f32 = 12.0;
pub const TEXT_SIZE_INPUT: f32 = 16.0;
pub const TEXT_SIZE_ERROR: f32 = 14.0;

pub const DIALOG_BUTTON_WIDTH: f32 = 100.0;

pub fn dialog_title<'a, Message>(
    title: impl ToString,
    style: style::DialogStyle,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(title).size(TEXT_SIZE_DIALOG_TITLE).style(style)).into()
}

pub fn dialog_button<'a, Message>(
    label: impl ToString,
    style: style::DialogButtonStyle,
) -> Button<'a, Message>
where
    Message: 'a,
{
    button(text(label).horizontal_alignment(alignment::Horizontal::Center))
        .width(DIALOG_BUTTON_WIDTH)
        .style(theme::Button::Custom(Box::new(style)))
}

pub fn button_bar<'a, Message, Renderer>() -> ButtonBar<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    ButtonBar {
        row: row![horizontal_space(Length::Fill)].spacing(SPACE_M),
    }
}

pub struct ButtonBar<'a, Message, Renderer> {
    row: Row<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> ButtonBar<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: iced::widget::button::StyleSheet,
{
    pub fn push(self, button: Button<'a, Message, Renderer>) -> Self {
        let Self { row } = self;
        Self {
            row: row.push(button),
        }
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

pub fn input_label<'a, Message>(label: impl ToString) -> Element<'a, Message> {
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
        input_label(label),
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

const ESCAPE: Event = Event::Keyboard(keyboard::Event::KeyPressed {
    key_code: keyboard::KeyCode::Escape,
    modifiers: keyboard::Modifiers::empty(),
});
const ENTER: Event = Event::Keyboard(keyboard::Event::KeyPressed {
    key_code: keyboard::KeyCode::Enter,
    modifiers: keyboard::Modifiers::empty(),
});

pub fn handle_ok_key<Message>(event: &Event, on_ok: Message) -> Option<Message> {
    if *event == ENTER {
        Some(on_ok)
    } else {
        None
    }
}

pub fn handle_cancel_key<Message>(event: &Event, on_cancel: Message) -> Option<Message> {
    if *event == ESCAPE {
        Some(on_cancel)
    } else {
        None
    }
}

pub fn handle_ok_and_cancel_keys<Message>(
    event: &Event,
    on_ok: Message,
    on_cancel: Message,
) -> Option<Message> {
    handle_ok_key(event, on_ok).or_else(|| handle_cancel_key(event, on_cancel))
}
