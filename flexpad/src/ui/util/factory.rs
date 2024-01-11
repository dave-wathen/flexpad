use flexpad_toolkit::prelude::*;
use iced::{keyboard, Color};
use rust_i18n::t;

pub const FLEXPAD_GRID_COLOR: Color = Color {
    r: 0.504,
    g: 0.699,
    b: 0.703,
    a: 1.0,
};

#[derive(Debug)]
pub enum FlexpadAction {
    Cancel,
    NewBlank,
    NewStarter,
    NewTextsheet,
    NewWorksheet,
    Ok,
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
            Self::Cancel => None,
            Self::NewBlank => Some('\u{E81B}'),
            Self::NewStarter => Some('\u{E81C}'),
            Self::NewTextsheet => Some('\u{E81E}'),
            Self::NewWorksheet => Some('\u{E81D}'),
            Self::Ok => None,
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
            Self::Cancel => Some(key(keyboard::KeyCode::Escape)),
            Self::NewBlank => Some(logo(key(keyboard::KeyCode::N))),
            Self::NewStarter => Some(shift(logo(key(keyboard::KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::Ok => Some(key(keyboard::KeyCode::Enter)),
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
            Self::Cancel => Some(key(keyboard::KeyCode::Escape)),
            Self::NewBlank => Some(ctrl(key(keyboard::KeyCode::N))),
            Self::NewStarter => Some(shift(ctrl(key(keyboard::KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::Ok => Some(key(keyboard::KeyCode::Enter)),
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
