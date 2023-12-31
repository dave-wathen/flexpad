use flexpad_toolkit::prelude::*;
use iced::{
    alignment, keyboard, theme,
    widget::{
        self, button, column, container, horizontal_space, row, text, tooltip, vertical_space,
        Button, Row, Text,
    },
    Color, Element, Event, Font, Length, Pixels,
};
use once_cell::sync::Lazy;
use rust_i18n::t;
use tracing::warn;

use crate::ui::style;

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
pub const TEXT_SIZE_TOOLTIP: Pixels = Pixels(12.0);

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

pub static ACTION_NEWBLANK: Lazy<Action> = Lazy::new(|| action("NewBlank"));
pub static ACTION_NEWSTARTER: Lazy<Action> = Lazy::new(|| action("NewStarter"));
pub static ACTION_NEWTEXTSHEET: Lazy<Action> = Lazy::new(|| action("NewTextsheet"));
pub static ACTION_NEWWORKSHEET: Lazy<Action> = Lazy::new(|| action("NewWorksheet"));
pub static ACTION_PADCLOSE: Lazy<Action> = Lazy::new(|| action("PadClose"));
pub static ACTION_PADDELETE: Lazy<Action> = Lazy::new(|| action("PadDelete"));
pub static ACTION_PADPROPERTIES: Lazy<Action> = Lazy::new(|| action("PadProperties"));
pub static ACTION_PRINT: Lazy<Action> = Lazy::new(|| action("Print"));
pub static ACTION_PROPERTIES: Lazy<Action> = Lazy::new(|| action("Properties"));
pub static ACTION_REDO: Lazy<Action> = Lazy::new(|| action("Redo"));
pub static ACTION_SHEETDELETE: Lazy<Action> = Lazy::new(|| action("SheetDelete"));
pub static ACTION_SHEETNEW: Lazy<Action> = Lazy::new(|| action("SheetNew"));
pub static ACTION_SHEETPROPERTIES: Lazy<Action> = Lazy::new(|| action("SheetProperties"));
pub static ACTION_UNDO: Lazy<Action> = Lazy::new(|| action("Undo"));

fn action(id: &str) -> Action {
    let mut result = Action::new(t!(&format!("Action.{id}.Name")));

    let full_i18n_name = |i18n_name| format!("{}.{}", rust_i18n::locale(), i18n_name);

    let i18n_name = format!("Action.{id}.ShortName");
    let short_name = t!(&i18n_name);
    if short_name != full_i18n_name(&i18n_name) {
        result = result.short_name(short_name)
    }

    let i18n_name = format!("Action.{id}.IconCodepoint");
    let codepoint = t!(&i18n_name);
    if codepoint != full_i18n_name(&i18n_name) {
        if codepoint.chars().count() == 1 {
            result = result.icon_codepoint(codepoint.chars().next().unwrap())
        } else {
            warn!("Invalid icon codepoint {}", i18n_name)
        };
    }

    let i18n_name = format!("Action.{id}.Shortcut");
    let shortcut = t!(&i18n_name);
    if shortcut != full_i18n_name(&i18n_name) {
        match shortcut.parse() {
            Ok(key) => result = result.shortcut(key),
            Err(_) => warn!("Invalid shortcut key {}", i18n_name),
        };
    }

    result
}

pub fn action_tooltip<'a, Message>(
    action: &Action,
    content: impl Into<Element<'a, Message, iced::Renderer>>,
    position: tooltip::Position,
) -> iced::widget::Tooltip<'a, Message, iced::Renderer> {
    let label = match action.shortcut {
        Some(key) => format!("{}  {}", action.name, key),
        None => action.name.clone(),
    };

    tooltip(content, label, position)
        .size(TEXT_SIZE_TOOLTIP)
        .style(theme::Container::Box)
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
