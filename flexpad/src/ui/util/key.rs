use iced::keyboard::{Event, KeyCode, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(pub Modifiers, pub KeyCode);

pub const fn key(code: KeyCode) -> Key {
    Key(Modifiers::empty(), code)
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.control() {
            write!(f, "\u{2303}")?;
        }
        if self.0.alt() {
            write!(f, "\u{2325}")?;
        }
        if self.0.shift() {
            write!(f, "\u{21E7}")?;
        }
        if self.0.command() {
            write!(f, "\u{2318}")?;
        }
        match self.1 {
            KeyCode::Key1 => write!(f, "1")?,
            KeyCode::Key2 => write!(f, "2")?,
            KeyCode::Key3 => write!(f, "3")?,
            KeyCode::Key4 => write!(f, "4")?,
            KeyCode::Key5 => write!(f, "5")?,
            KeyCode::Key6 => write!(f, "6")?,
            KeyCode::Key7 => write!(f, "7")?,
            KeyCode::Key8 => write!(f, "8")?,
            KeyCode::Key9 => write!(f, "9")?,
            KeyCode::Key0 => write!(f, "0")?,
            KeyCode::A => write!(f, "A")?,
            KeyCode::B => write!(f, "B")?,
            KeyCode::C => write!(f, "C")?,
            KeyCode::D => write!(f, "D")?,
            KeyCode::E => write!(f, "E")?,
            KeyCode::F => write!(f, "F")?,
            KeyCode::G => write!(f, "G")?,
            KeyCode::H => write!(f, "H")?,
            KeyCode::I => write!(f, "I")?,
            KeyCode::J => write!(f, "J")?,
            KeyCode::K => write!(f, "K")?,
            KeyCode::L => write!(f, "L")?,
            KeyCode::M => write!(f, "M")?,
            KeyCode::N => write!(f, "N")?,
            KeyCode::O => write!(f, "O")?,
            KeyCode::P => write!(f, "P")?,
            KeyCode::Q => write!(f, "Q")?,
            KeyCode::R => write!(f, "R")?,
            KeyCode::S => write!(f, "S")?,
            KeyCode::T => write!(f, "T")?,
            KeyCode::U => write!(f, "U")?,
            KeyCode::V => write!(f, "V")?,
            KeyCode::W => write!(f, "W")?,
            KeyCode::X => write!(f, "X")?,
            KeyCode::Y => write!(f, "Y")?,
            KeyCode::Z => write!(f, "Z")?,
            KeyCode::Escape => write!(f, "\u{238B}")?,
            KeyCode::F1 => write!(f, "F1")?,
            KeyCode::F2 => write!(f, "F2")?,
            KeyCode::F3 => write!(f, "F3")?,
            KeyCode::F4 => write!(f, "F4")?,
            KeyCode::F5 => write!(f, "F5")?,
            KeyCode::F6 => write!(f, "F6")?,
            KeyCode::F7 => write!(f, "F7")?,
            KeyCode::F8 => write!(f, "F8")?,
            KeyCode::F9 => write!(f, "F9")?,
            KeyCode::F10 => write!(f, "F10")?,
            KeyCode::F11 => write!(f, "F11")?,
            KeyCode::F12 => write!(f, "F12")?,
            KeyCode::F13 => write!(f, "F13")?,
            KeyCode::F14 => write!(f, "F14")?,
            KeyCode::F15 => write!(f, "F15")?,
            KeyCode::F16 => write!(f, "F16")?,
            KeyCode::F17 => write!(f, "F17")?,
            KeyCode::F18 => write!(f, "F18")?,
            KeyCode::F19 => write!(f, "F19")?,
            KeyCode::F20 => write!(f, "F20")?,
            KeyCode::F21 => write!(f, "F21")?,
            KeyCode::F22 => write!(f, "F22")?,
            KeyCode::F23 => write!(f, "F23")?,
            KeyCode::F24 => write!(f, "F24")?,
            KeyCode::Snapshot => write!(f, "\u{2399}")?,
            KeyCode::Scroll => write!(f, "\u{21F3}")?,
            KeyCode::Pause => write!(f, "\u{23F8}")?,
            KeyCode::Insert => write!(f, "Insert")?,
            KeyCode::Home => write!(f, "\u{2196}")?,
            KeyCode::Delete => write!(f, "\u{2326}")?,
            KeyCode::End => write!(f, "\u{2198}")?,
            KeyCode::PageDown => write!(f, "\u{21DF}")?,
            KeyCode::PageUp => write!(f, "\u{21DE}")?,
            KeyCode::Left => write!(f, "\u{23F4}")?,
            KeyCode::Up => write!(f, "\u{23F6}")?,
            KeyCode::Right => write!(f, "\u{23F5}")?,
            KeyCode::Down => write!(f, "\u{23F7}")?,
            KeyCode::Backspace => write!(f, "\u{232B}")?,
            KeyCode::Enter => write!(f, "\u{21A9}")?,
            KeyCode::Space => write!(f, "Space")?,
            KeyCode::Compose => write!(f, "\u{2384}")?,
            KeyCode::Caret => write!(f, "^")?,
            KeyCode::Numlock => write!(f, "\u{21ED}")?,
            KeyCode::Numpad0 => write!(f, "1")?,
            KeyCode::Numpad1 => write!(f, "2")?,
            KeyCode::Numpad2 => write!(f, "3")?,
            KeyCode::Numpad3 => write!(f, "4")?,
            KeyCode::Numpad4 => write!(f, "5")?,
            KeyCode::Numpad5 => write!(f, "6")?,
            KeyCode::Numpad6 => write!(f, "7")?,
            KeyCode::Numpad7 => write!(f, "8")?,
            KeyCode::Numpad8 => write!(f, "9")?,
            KeyCode::Numpad9 => write!(f, "0")?,
            KeyCode::NumpadAdd => write!(f, "+")?,
            KeyCode::NumpadDivide => write!(f, "/")?,
            KeyCode::NumpadDecimal => write!(f, ".")?,
            KeyCode::NumpadComma => write!(f, ",")?,
            KeyCode::NumpadEnter => write!(f, "\u{21A9}")?,
            KeyCode::NumpadEquals => write!(f, "=")?,
            KeyCode::NumpadMultiply => write!(f, "*")?,
            KeyCode::NumpadSubtract => write!(f, "-")?,
            KeyCode::AbntC1 => write!(f, "AbntC1")?,
            KeyCode::AbntC2 => write!(f, "AbntC2")?,
            KeyCode::Apostrophe => write!(f, "'")?,
            KeyCode::Apps => write!(f, "\u{2630}")?,
            KeyCode::Asterisk => write!(f, "*")?,
            KeyCode::At => write!(f, "@")?,
            KeyCode::Ax => write!(f, "Ax")?,
            KeyCode::Backslash => write!(f, "\\")?,
            KeyCode::Calculator => write!(f, "Calc")?,
            KeyCode::Capital => write!(f, "Capital")?,
            KeyCode::Colon => write!(f, ":")?,
            KeyCode::Comma => write!(f, "',")?,
            KeyCode::Convert => write!(f, "Convert")?,
            KeyCode::Equals => write!(f, "-=")?,
            KeyCode::Grave => write!(f, "`")?,
            KeyCode::Kana => write!(f, "Kana")?,
            KeyCode::Kanji => write!(f, "Kanji")?,
            KeyCode::LAlt => write!(f, "\u{2325}")?,
            KeyCode::LBracket => write!(f, "[")?,
            KeyCode::LControl => write!(f, "\u{2303}")?,
            KeyCode::LShift => write!(f, "\u{21E7}")?,
            KeyCode::LWin => write!(f, "\u{2318}")?,
            KeyCode::Mail => write!(f, "Mail")?,
            KeyCode::MediaSelect => write!(f, "MediaSelect")?,
            KeyCode::MediaStop => write!(f, "MediaStop")?,
            KeyCode::Minus => write!(f, "-")?,
            KeyCode::Mute => write!(f, "Mute")?,
            KeyCode::MyComputer => write!(f, "MyComputer")?,
            KeyCode::NavigateForward => write!(f, "Forward")?,
            KeyCode::NavigateBackward => write!(f, "Back")?,
            KeyCode::NextTrack => write!(f, "NextTrack")?,
            KeyCode::NoConvert => write!(f, "NoConvert")?,
            KeyCode::OEM102 => write!(f, "OEM102")?,
            KeyCode::Period => write!(f, ".")?,
            KeyCode::PlayPause => write!(f, "PlayPause")?,
            KeyCode::Plus => write!(f, "+")?,
            KeyCode::Power => write!(f, "\u{2318}")?,
            KeyCode::PrevTrack => todo!(),
            KeyCode::RAlt => write!(f, "\u{2325}")?,
            KeyCode::RBracket => write!(f, "]")?,
            KeyCode::RControl => write!(f, "\u{2303}")?,
            KeyCode::RShift => write!(f, "\u{21E7}")?,
            KeyCode::RWin => write!(f, "\u{2318}")?,
            KeyCode::Semicolon => write!(f, ";")?,
            KeyCode::Slash => write!(f, "/")?,
            KeyCode::Sleep => write!(f, "Sleep")?,
            KeyCode::Stop => write!(f, "Stop")?,
            KeyCode::Sysrq => write!(f, "Sysrq")?,
            KeyCode::Tab => write!(f, "\u{21E5}")?,
            KeyCode::Underline => write!(f, "Sysrq")?,
            KeyCode::Unlabeled => write!(f, "Unlabeled")?,
            KeyCode::VolumeDown => write!(f, "VolumeDown")?,
            KeyCode::VolumeUp => write!(f, "VolumeUp")?,
            KeyCode::Wake => write!(f, "Wake")?,
            KeyCode::WebBack => write!(f, "WebBack")?,
            KeyCode::WebFavorites => write!(f, "WebFavorites")?,
            KeyCode::WebForward => write!(f, "WebForward")?,
            KeyCode::WebHome => write!(f, "WebHome")?,
            KeyCode::WebRefresh => write!(f, "WebRefresh")?,
            KeyCode::WebSearch => write!(f, "WebSearch")?,
            KeyCode::WebStop => write!(f, "WebStop")?,
            KeyCode::Yen => write!(f, "WebStop")?,
            KeyCode::Copy => write!(f, "Copy")?,
            KeyCode::Paste => write!(f, "Paste")?,
            KeyCode::Cut => write!(f, "Cut")?,
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub const fn shift(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::SHIFT.bits());
    Key(modifiers, key.1)
}

pub const fn ctrl(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::CTRL.bits());
    Key(modifiers, key.1)
}

#[allow(dead_code)]
pub const fn alt(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::ALT.bits());
    Key(modifiers, key.1)
}

#[allow(dead_code)]
pub const fn command(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::COMMAND.bits());
    Key(modifiers, key.1)
}

pub const fn pressed(key: Key) -> Event {
    Event::KeyPressed {
        key_code: key.1,
        modifiers: key.0,
    }
}

pub const fn released(key: Key) -> Event {
    Event::KeyReleased {
        key_code: key.1,
        modifiers: key.0,
    }
}
