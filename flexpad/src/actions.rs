use flexpad_toolkit::prelude::*;
use iced::keyboard::KeyCode;
use rust_i18n::t;

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
            Self::Cancel => Some(key(KeyCode::Escape)),
            Self::NewBlank => Some(logo(key(KeyCode::N))),
            Self::NewStarter => Some(shift(logo(key(KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::Ok => Some(key(KeyCode::Enter)),
            Self::PadDelete => Some(logo(key(KeyCode::Delete))),
            Self::PadClose => Some(logo(key(KeyCode::W))),
            Self::PadProperties => Some(logo(key(KeyCode::Comma))),
            Self::Print => Some(logo(key(KeyCode::P))),
            Self::Properties => None,
            Self::Redo => Some(shift(logo(key(KeyCode::Z)))),
            Self::SheetDelete => Some(alt(key(KeyCode::Delete))),
            Self::SheetNew => Some(alt(key(KeyCode::N))),
            Self::SheetProperties => Some(alt(key(KeyCode::Comma))),
            Self::Undo => Some(logo(key(KeyCode::Z))),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn shortcut(&self) -> Option<Key> {
        match self {
            Self::Cancel => Some(key(KeyCode::Escape)),
            Self::NewBlank => Some(ctrl(key(KeyCode::N))),
            Self::NewStarter => Some(shift(ctrl(key(KeyCode::N)))),
            Self::NewTextsheet => None,
            Self::NewWorksheet => None,
            Self::Ok => Some(key(KeyCode::Enter)),
            Self::PadDelete => Some(ctrl(key(KeyCode::Delete))),
            Self::PadClose => Some(ctrl(key(KeyCode::W))),
            Self::PadProperties => Some(ctrl(key(KeyCode::Comma))),
            Self::Print => Some(ctrl(key(KeyCode::P))),
            Self::Properties => None,
            Self::Redo => Some(shift(ctrl(key(KeyCode::Z)))),
            Self::SheetDelete => Some(alt(key(KeyCode::Delete))),
            Self::SheetNew => Some(alt(key(KeyCode::N))),
            Self::SheetProperties => Some(alt(key(KeyCode::Comma))),
            Self::Undo => Some(ctrl(key(KeyCode::Z))),
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
