use flexpad_toolkit::prelude::*;
use iced::{
    alignment, keyboard, theme,
    widget::{
        self, button, column, container, horizontal_space, row, text, vertical_space, Button, Row,
        Text,
    },
    Color, Element, Event, Font, Length, Pixels,
};
use rust_i18n::t;

pub const FLEXPAD_GRID_COLOR: Color = Color {
    r: 0.504,
    g: 0.699,
    b: 0.703,
    a: 1.0,
};

pub const TEXT_SIZE_APP_TITLE: Pixels = Pixels(20.0);
pub const TEXT_SIZE_DIALOG_TITLE: Pixels = Pixels(16.0);
pub const TEXT_SIZE_LABEL: Pixels = Pixels(12.0);
pub const TEXT_SIZE_INPUT: Pixels = Pixels(16.0);
pub const TEXT_SIZE_ERROR: Pixels = Pixels(14.0);

pub const ICON_BUTTON_SIZE: Pixels = Pixels(48.0);

pub const DIALOG_BUTTON_WIDTH: f32 = 100.0;

pub const ICON_FX: char = '\u{E81A}';
pub const ICON_OPEN_DOWN: char = '\u{E806}';

pub fn icon<'a>(codepoint: char, size: impl Into<Pixels>) -> Text<'a, iced::Renderer> {
    const ICON_FONT: Font = Font::with_name("flexpad-icons");

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

#[derive(Debug)]
pub enum FlexpadAction {
    NewBlank,
    NewStarter,
    NewTextsheet,
    NewWorksheet,
    PadClose,
    PadDelete,
    PadProperties,
    Print,
    Properties,
    Redo,
    SheetDelete,
    SheetNew,
    SheetProperties,
    Undo,
}

impl FlexpadAction {
    fn icon_codepoint(&self) -> Option<char> {
        match self {
            Self::NewBlank => Some('\u{E81B}'),
            Self::NewStarter => Some('\u{E81C}'),
            Self::NewTextsheet => Some('\u{E81E}'),
            Self::NewWorksheet => Some('\u{E81D}'),
            Self::PadDelete => None,
            Self::PadClose => None,
            Self::PadProperties => None,
            Self::Print => Some('\u{E807}'),
            Self::Properties => Some('\u{E808}'),
            Self::Redo => Some('\u{E800}'),
            Self::SheetDelete => None,
            Self::SheetNew => None,
            Self::SheetProperties => None,
            Self::Undo => Some('\u{E801}'),
        }
    }

    #[cfg(target_os = "macos")]
    fn shortcut(&self) -> Option<Key> {
        match self {
            Self::NewBlank => Some(logo(key(keyboard::KeyCode::N))),
            Self::NewStarter => Some(shift(logo(key(keyboard::KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::PadDelete => Some(logo(key(keyboard::KeyCode::Delete))),
            Self::PadClose => Some(logo(key(keyboard::KeyCode::W))),
            Self::PadProperties => Some(logo(key(keyboard::KeyCode::Comma))),
            Self::Print => Some(logo(key(keyboard::KeyCode::P))),
            Self::Properties => None,
            Self::Redo => Some(shift(logo(key(keyboard::KeyCode::Z)))),
            Self::SheetDelete => Some(alt(key(keyboard::KeyCode::Delete))),
            Self::SheetNew => Some(alt(key(keyboard::KeyCode::N))),
            Self::SheetProperties => Some(alt(key(keyboard::KeyCode::Comma))),
            Self::Undo => Some(logo(key(keyboard::KeyCode::Z))),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn shortcut(&self) -> Option<Key> {
        match self {
            Self::NewBlank => Some(ctrl(key(keyboard::KeyCode::N))),
            Self::NewStarter => Some(shift(ctrl(key(keyboard::KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::PadDelete => Some(ctrl(key(keyboard::KeyCode::Delete))),
            Self::PadClose => Some(ctrl(key(keyboard::KeyCode::W))),
            Self::PadProperties => Some(ctrl(key(keyboard::KeyCode::Comma))),
            Self::Print => Some(ctrl(key(keyboard::KeyCode::P))),
            Self::Properties => None,
            Self::Redo => Some(shift(ctrl(key(keyboard::KeyCode::Z)))),
            Self::SheetDelete => Some(alt(key(keyboard::KeyCode::Delete))),
            Self::SheetNew => Some(alt(key(keyboard::KeyCode::N))),
            Self::SheetProperties => Some(alt(key(keyboard::KeyCode::Comma))),
            Self::Undo => Some(ctrl(key(keyboard::KeyCode::Z))),
        }
    }
}

impl std::fmt::Display for FlexpadAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl From<FlexpadAction> for Action {
    fn from(value: FlexpadAction) -> Self {
        let full_i18n_name = |i18n_name| format!("{}.{}", rust_i18n::locale(), i18n_name);
        let id = value.to_string();

        let i18n_name = format!("Action.{id}.Name");
        let mut result = Action::new(t!(&i18n_name));

        let i18n_name = format!("Action.{id}.ShortName");
        let short_name = t!(&i18n_name);
        if short_name != full_i18n_name(&i18n_name) {
            result = result.short_name(short_name)
        }

        if let Some(codepoint) = value.icon_codepoint() {
            result = result.icon_codepoint(codepoint);
        }

        if let Some(shortcut) = value.shortcut() {
            result = result.shortcut(shortcut);
        }

        result
    }
}

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
