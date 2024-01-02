use iced::keyboard::{Event, KeyCode, Modifiers};
use once_cell::sync::Lazy;
use std::{collections::HashMap, error::Error, fmt, str::FromStr};

static KEY_DB: Lazy<KeyDatabase> = Lazy::new(KeyDatabase::new);

/// A Key, or key combination.  Keys are turned into their canonical representations.
/// That is all number pad keys are translated to their main keyboard equivalents and
/// duplicated modifier keys (Control, Alt, Shift, Logo) are represented using their
/// left-hand versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(Modifiers, KeyCode);

impl Key {
    /// Create a new key from a given Iced [`Modifiers`] and [`KeyCode`] .
    pub fn new(modifiers: Modifiers, code: KeyCode) -> Self {
        let mut result = key(code);
        if modifiers.shift() {
            result = shift(result);
        }
        if modifiers.control() {
            result = ctrl(result);
        }
        if modifiers.alt() {
            result = alt(result);
        }
        if modifiers.logo() {
            result = logo(result);
        }
        result
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.control() {
            f.write_str(KEY_DB.key_code_symbol(KeyCode::LControl))?;
        }
        if self.0.alt() {
            f.write_str(KEY_DB.key_code_symbol(KeyCode::LAlt))?;
        }
        if self.0.shift() {
            f.write_str(KEY_DB.key_code_symbol(KeyCode::LShift))?;
        }
        if self.0.logo() {
            f.write_str(KEY_DB.key_code_symbol(KeyCode::LWin))?;
        }
        f.write_str(KEY_DB.key_code_symbol(self.1))
    }
}

/// Create a [`Key`] from a string representation.  The string representation can be
/// the same as Display string of the Key or it can use synonyms for certain keys,
/// including modifier keys.  When modifier key synonyms are used the parts must be
/// separated using hyphens.  Examples:
///
///  * Home
///  * Ctrl-End
///  * Alt-Shift-A
///  * Command-Escape
///  * Win-Shift-N
impl FromStr for Key {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match KEY_DB.match_start(s) {
            Some(m) => {
                let remainder = &s[m.matched.len()..];
                if remainder.is_empty() {
                    Ok(key(m.key_code))
                } else if is_modifier(m.key_code) {
                    let remainder = match m.kind {
                        MatchKind::Symbol => remainder,
                        MatchKind::Synonym => {
                            if !starts_with_ignore_case(remainder, "-") {
                                return Err(KeyParseError);
                            }
                            &remainder[1..]
                        }
                    };

                    match m.key_code {
                        KeyCode::LControl => Ok(ctrl(remainder.parse()?)),
                        KeyCode::LAlt => Ok(alt(remainder.parse()?)),
                        KeyCode::LShift => Ok(shift(remainder.parse()?)),
                        KeyCode::LWin => Ok(logo(remainder.parse()?)),
                        _ => unreachable!(),
                    }
                } else {
                    Err(KeyParseError)
                }
            }
            None => Err(KeyParseError),
        }
    }
}

/// Create a [`Key`] from a given Iced [`KeyCode`].
pub const fn key(code: KeyCode) -> Key {
    Key(Modifiers::empty(), canonical(code))
}

/// Create a [`Key`] as the shift-modified version of the given [`Key`].
pub const fn shift(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::SHIFT.bits());
    Key(modifiers, key.1)
}

/// Create a [`Key`] as the control-modified versiopn of the given [`Key`].
pub const fn ctrl(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::CTRL.bits());
    Key(modifiers, key.1)
}

/// Create a [`Key`] as the alt-modified versiopn of the given [`Key`].
pub const fn alt(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::ALT.bits());
    Key(modifiers, key.1)
}

/// Create a [`Key`] as the logo-modified versiopn of the given [`Key`].
pub const fn logo(key: Key) -> Key {
    let modifiers = Modifiers::from_bits_truncate(key.0.bits() | Modifiers::LOGO.bits());
    Key(modifiers, key.1)
}

/// Create an Iced key pressed event for the given [`Key`].
pub const fn pressed(key: Key) -> Event {
    Event::KeyPressed {
        key_code: key.1,
        modifiers: key.0,
    }
}

/// Create an Iced key released event for the given [`Key`].
pub const fn released(key: Key) -> Event {
    Event::KeyReleased {
        key_code: key.1,
        modifiers: key.0,
    }
}

#[derive(Debug)]
pub struct KeyParseError;

impl fmt::Display for KeyParseError {
    #[allow(deprecated, deprecated_in_future)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for KeyParseError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "invalid key value"
    }
}

#[cfg(target_os = "macos")]
const LOGO_SYMBOL: &str = "⌘";
#[cfg(target_os = "macos")]
const ALT_SYMBOL: &str = "⌥";

#[cfg(target_os = "windows")]
const LOGO_SYMBOL: &str = "⊞";
#[cfg(target_os = "windows")]
const ALT_SYMBOL: &str = "⎇";

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const LOGO_SYMBOL: &str = "❖";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const ALT_SYMBOL: &str = "⎇";

const SHIFT_SYMBOL: &str = "⇧";
const CONTROL_SYMBOL: &str = "⌃";

struct KeyDatabase {
    key_code_to_symbol: HashMap<KeyCode, &'static str>,
    text_to_key_code: Vec<(&'static str, KeyCode, MatchKind)>,
}

impl KeyDatabase {
    fn new() -> Self {
        let mut result = Self {
            key_code_to_symbol: Default::default(),
            text_to_key_code: Default::default(),
        };

        let mut add_key = |key_data: KeyData| {
            result
                .key_code_to_symbol
                .insert(key_data.key_code, key_data.symbol);

            result
                .text_to_key_code
                .push((key_data.symbol, key_data.key_code, MatchKind::Symbol));

            for synonym in key_data.synonyms {
                result
                    .text_to_key_code
                    .push((synonym, key_data.key_code, MatchKind::Synonym));
            }
        };

        add_key(
            KeyData::new(KeyCode::LControl, CONTROL_SYMBOL)
                .synonym("Control")
                .synonym("Ctrl"),
        );
        add_key(KeyData::new(KeyCode::LAlt, ALT_SYMBOL).synonym("Alt"));
        add_key(KeyData::new(KeyCode::LShift, SHIFT_SYMBOL).synonym("Shift"));
        add_key(
            KeyData::new(KeyCode::LWin, LOGO_SYMBOL)
                .synonym("Command")
                .synonym("Cmd")
                .synonym("Logo")
                .synonym("Super")
                .synonym("Win"),
        );

        add_key(KeyData::new(KeyCode::Key0, "0"));
        add_key(KeyData::new(KeyCode::Key1, "1"));
        add_key(KeyData::new(KeyCode::Key2, "2"));
        add_key(KeyData::new(KeyCode::Key3, "3"));
        add_key(KeyData::new(KeyCode::Key4, "4"));
        add_key(KeyData::new(KeyCode::Key5, "5"));
        add_key(KeyData::new(KeyCode::Key6, "6"));
        add_key(KeyData::new(KeyCode::Key7, "7"));
        add_key(KeyData::new(KeyCode::Key8, "8"));
        add_key(KeyData::new(KeyCode::Key9, "9"));
        add_key(KeyData::new(KeyCode::A, "A"));
        add_key(KeyData::new(KeyCode::AbntC1, "AbntC1"));
        add_key(KeyData::new(KeyCode::AbntC2, "AbntC2"));
        add_key(KeyData::new(KeyCode::Apostrophe, "'"));
        add_key(KeyData::new(KeyCode::Apps, "☰").synonym("Apps"));
        add_key(KeyData::new(KeyCode::Asterisk, "*"));
        add_key(KeyData::new(KeyCode::At, "@"));
        add_key(KeyData::new(KeyCode::Ax, "Ax"));
        add_key(KeyData::new(KeyCode::B, "B"));
        add_key(KeyData::new(KeyCode::Backslash, "\\"));
        add_key(KeyData::new(KeyCode::Backspace, "⌫").synonym("Backspace"));
        add_key(
            KeyData::new(KeyCode::Calculator, "Calc")
                .synonym("Calculator")
                .synonym("Calc"),
        );
        add_key(KeyData::new(KeyCode::C, "C"));
        add_key(KeyData::new(KeyCode::Capital, "Capital"));
        add_key(KeyData::new(KeyCode::Caret, "^"));
        add_key(KeyData::new(KeyCode::Colon, ":"));
        add_key(KeyData::new(KeyCode::Comma, ","));
        add_key(KeyData::new(KeyCode::Compose, "⎄").synonym("Compose"));
        add_key(KeyData::new(KeyCode::Convert, "Convert"));
        add_key(KeyData::new(KeyCode::Copy, "Copy").synonym("Copy"));
        add_key(KeyData::new(KeyCode::Cut, "Cut").synonym("Cut"));
        add_key(KeyData::new(KeyCode::D, "D"));
        add_key(
            KeyData::new(KeyCode::Delete, "⌦")
                .synonym("Delete")
                .synonym("Del"),
        );
        add_key(KeyData::new(KeyCode::Down, "⏷").synonym("Down"));
        add_key(KeyData::new(KeyCode::E, "E"));
        add_key(KeyData::new(KeyCode::End, "↘").synonym("End"));
        add_key(
            KeyData::new(KeyCode::Enter, "↩")
                .synonym("Enter")
                .synonym("Return"),
        );
        add_key(KeyData::new(KeyCode::Equals, "="));
        add_key(
            KeyData::new(KeyCode::Escape, "⎋")
                .synonym("Escape")
                .synonym("Esc"),
        );
        add_key(KeyData::new(KeyCode::F, "F"));
        add_key(KeyData::new(KeyCode::F1, "F1"));
        add_key(KeyData::new(KeyCode::F2, "F2"));
        add_key(KeyData::new(KeyCode::F3, "F3"));
        add_key(KeyData::new(KeyCode::F4, "F4"));
        add_key(KeyData::new(KeyCode::F5, "F5"));
        add_key(KeyData::new(KeyCode::F6, "F6"));
        add_key(KeyData::new(KeyCode::F7, "F7"));
        add_key(KeyData::new(KeyCode::F8, "F8"));
        add_key(KeyData::new(KeyCode::F9, "F9"));
        add_key(KeyData::new(KeyCode::F10, "F10"));
        add_key(KeyData::new(KeyCode::F11, "F11"));
        add_key(KeyData::new(KeyCode::F12, "F12"));
        add_key(KeyData::new(KeyCode::F13, "F13"));
        add_key(KeyData::new(KeyCode::F14, "F14"));
        add_key(KeyData::new(KeyCode::F15, "F15"));
        add_key(KeyData::new(KeyCode::F16, "F16"));
        add_key(KeyData::new(KeyCode::F17, "F17"));
        add_key(KeyData::new(KeyCode::F18, "F18"));
        add_key(KeyData::new(KeyCode::F19, "F19"));
        add_key(KeyData::new(KeyCode::F20, "F20"));
        add_key(KeyData::new(KeyCode::F21, "F21"));
        add_key(KeyData::new(KeyCode::F22, "F22"));
        add_key(KeyData::new(KeyCode::F23, "F23"));
        add_key(KeyData::new(KeyCode::F24, "F24"));
        add_key(KeyData::new(KeyCode::G, "G"));
        add_key(KeyData::new(KeyCode::Grave, "`"));
        add_key(KeyData::new(KeyCode::H, "H"));
        add_key(KeyData::new(KeyCode::Home, "↖").synonym("Home"));
        add_key(KeyData::new(KeyCode::I, "I"));
        add_key(
            KeyData::new(KeyCode::Insert, "Insert")
                .synonym("Insert")
                .synonym("Ins"),
        );
        add_key(KeyData::new(KeyCode::J, "J"));
        add_key(KeyData::new(KeyCode::K, "K"));
        add_key(KeyData::new(KeyCode::Kana, "Kana"));
        add_key(KeyData::new(KeyCode::Kanji, "Kanji"));
        add_key(KeyData::new(KeyCode::L, "L"));
        add_key(KeyData::new(KeyCode::LBracket, "["));
        add_key(KeyData::new(KeyCode::Left, "⏴").synonym("Left"));
        add_key(KeyData::new(KeyCode::M, "M"));
        add_key(KeyData::new(KeyCode::Mail, "Mail").synonym("Mail"));
        add_key(KeyData::new(KeyCode::MediaSelect, "MediaSelect").synonym("MediaSelect"));
        add_key(KeyData::new(KeyCode::MediaStop, "MediaStop").synonym("MediaStop"));
        add_key(KeyData::new(KeyCode::Minus, "-"));
        add_key(KeyData::new(KeyCode::Mute, "Mute").synonym("Mute"));
        add_key(KeyData::new(KeyCode::MyComputer, "MyComputer").synonym("MyComputer"));
        add_key(KeyData::new(KeyCode::N, "N"));
        add_key(
            KeyData::new(KeyCode::NavigateBackward, "Backward")
                .synonym("Backward")
                .synonym("Prior"),
        );
        add_key(
            KeyData::new(KeyCode::NavigateForward, "Forward")
                .synonym("Forward")
                .synonym("Next"),
        );
        add_key(KeyData::new(KeyCode::NextTrack, "⏭").synonym("NextTrack"));
        add_key(KeyData::new(KeyCode::NoConvert, "NoConvert").synonym("NoConvert"));
        add_key(KeyData::new(KeyCode::Numlock, "⇭").synonym("NumLock"));
        add_key(KeyData::new(KeyCode::O, "O"));
        add_key(KeyData::new(KeyCode::OEM102, "EOM102"));
        add_key(KeyData::new(KeyCode::P, "P"));
        add_key(KeyData::new(KeyCode::PageDown, "⇟").synonym("PageDown"));
        add_key(KeyData::new(KeyCode::PageUp, "⇞").synonym("PageUp"));
        add_key(KeyData::new(KeyCode::Paste, "Paste"));
        add_key(KeyData::new(KeyCode::Pause, "⏸").synonym("Pause"));
        add_key(KeyData::new(KeyCode::Period, "."));
        add_key(KeyData::new(KeyCode::PlayPause, "⏯").synonym("PlayPause"));
        add_key(KeyData::new(KeyCode::Plus, "+"));
        add_key(KeyData::new(KeyCode::Power, "⏻").synonym("Power"));
        add_key(
            KeyData::new(KeyCode::PrevTrack, "⏮")
                .synonym("PreviousTrack")
                .synonym("PrevTrack"),
        );
        add_key(KeyData::new(KeyCode::Q, "Q"));
        add_key(KeyData::new(KeyCode::R, "R"));
        add_key(KeyData::new(KeyCode::RBracket, "]"));
        add_key(KeyData::new(KeyCode::Right, "⏵").synonym("Right"));
        add_key(KeyData::new(KeyCode::S, "S"));
        add_key(KeyData::new(KeyCode::Scroll, "⇳").synonym("Scroll"));
        add_key(KeyData::new(KeyCode::Semicolon, ";"));
        add_key(KeyData::new(KeyCode::Slash, "/"));
        add_key(KeyData::new(KeyCode::Sleep, "⏾").synonym("Sleep"));
        add_key(KeyData::new(KeyCode::Snapshot, "⎙").synonym("Snapshot"));
        add_key(KeyData::new(KeyCode::Space, "Space").synonym("Space"));
        add_key(KeyData::new(KeyCode::Stop, "⏹").synonym("Stop"));
        add_key(KeyData::new(KeyCode::Sysrq, "SysRq"));
        add_key(KeyData::new(KeyCode::T, "T"));
        add_key(KeyData::new(KeyCode::Tab, "⇥").synonym("Tab"));
        add_key(KeyData::new(KeyCode::U, "U"));
        add_key(KeyData::new(KeyCode::Underline, "_"));
        add_key(KeyData::new(KeyCode::Unlabeled, "Unlabeled"));
        add_key(KeyData::new(KeyCode::Up, "⏶").synonym("Up"));
        add_key(KeyData::new(KeyCode::V, "V"));
        add_key(KeyData::new(KeyCode::VolumeDown, "VolumeDown").synonym("VolumeDown"));
        add_key(KeyData::new(KeyCode::VolumeUp, "VolumeUp").synonym("VolumeUp"));
        add_key(KeyData::new(KeyCode::W, "W"));
        add_key(KeyData::new(KeyCode::Wake, "Wake").synonym("Wake"));
        add_key(KeyData::new(KeyCode::WebBack, "WebBack").synonym("WebBack"));
        add_key(KeyData::new(KeyCode::WebFavorites, "WebFavorites").synonym("WebFavorites"));
        add_key(KeyData::new(KeyCode::WebForward, "WebForward").synonym("WebForward"));
        add_key(KeyData::new(KeyCode::WebHome, "WebHome").synonym("WebHome"));
        add_key(KeyData::new(KeyCode::WebRefresh, "WebRefresh").synonym("WebRefresh"));
        add_key(KeyData::new(KeyCode::WebSearch, "WebSearch").synonym("WebSearch"));
        add_key(KeyData::new(KeyCode::WebStop, "WebStop").synonym("WebStop"));
        add_key(KeyData::new(KeyCode::X, "X"));
        add_key(KeyData::new(KeyCode::Y, "Y"));
        add_key(KeyData::new(KeyCode::Yen, "¥").synonym("Yen"));
        add_key(KeyData::new(KeyCode::Z, "Z"));

        result
    }

    fn key_code_symbol(&self, key_code: KeyCode) -> &str {
        self.key_code_to_symbol
            .get(&canonical(key_code))
            .unwrap_or_else(|| panic!("KeyDatabase is missing KeyCode: {:?}", key_code))
    }

    fn match_start(&self, text: &str) -> Option<Match> {
        self.text_to_key_code
            .iter()
            .filter(|(txt, _, _)| starts_with_ignore_case(text, txt))
            .reduce(|pair1, pair2| {
                if pair1.0.len() > pair2.0.len() {
                    pair1
                } else {
                    pair2
                }
            })
            .cloned()
            .map(|(matched, key_code, kind)| Match {
                matched,
                key_code,
                kind,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchKind {
    Symbol,
    Synonym,
}

struct Match {
    matched: &'static str,
    key_code: KeyCode,
    kind: MatchKind,
}

struct KeyData {
    key_code: KeyCode,
    symbol: &'static str,
    synonyms: Vec<&'static str>,
}

impl KeyData {
    fn new(key_code: KeyCode, symbol: &'static str) -> Self {
        Self {
            key_code,
            symbol,
            synonyms: vec![],
        }
    }

    fn synonym(mut self, synonym: &'static str) -> Self {
        self.synonyms.push(synonym);
        self
    }
}

const fn is_modifier(key_code: KeyCode) -> bool {
    matches!(
        key_code,
        KeyCode::LControl | KeyCode::LAlt | KeyCode::LShift | KeyCode::LWin
    )
}

const fn canonical(key_code: KeyCode) -> KeyCode {
    match key_code {
        KeyCode::Numpad0 => KeyCode::Key0,
        KeyCode::Numpad1 => KeyCode::Key1,
        KeyCode::Numpad2 => KeyCode::Key2,
        KeyCode::Numpad3 => KeyCode::Key3,
        KeyCode::Numpad4 => KeyCode::Key4,
        KeyCode::Numpad5 => KeyCode::Key5,
        KeyCode::Numpad6 => KeyCode::Key6,
        KeyCode::Numpad7 => KeyCode::Key7,
        KeyCode::Numpad8 => KeyCode::Key8,
        KeyCode::Numpad9 => KeyCode::Key9,
        KeyCode::NumpadAdd => KeyCode::Plus,
        KeyCode::NumpadDivide => KeyCode::Slash,
        KeyCode::NumpadDecimal => KeyCode::Period,
        KeyCode::NumpadComma => KeyCode::Comma,
        KeyCode::NumpadEnter => KeyCode::Enter,
        KeyCode::NumpadEquals => KeyCode::Equals,
        KeyCode::NumpadMultiply => KeyCode::Asterisk,
        KeyCode::NumpadSubtract => KeyCode::Minus,
        KeyCode::RAlt => KeyCode::LAlt,
        KeyCode::RControl => KeyCode::LControl,
        KeyCode::RShift => KeyCode::LShift,
        KeyCode::RWin => KeyCode::LWin,
        other => other,
    }
}

fn starts_with_ignore_case(s: &str, sub: &str) -> bool {
    let mut s_chars = s.chars();
    for sub_ch in sub.chars() {
        match s_chars.next() {
            Some(s_ch) => {
                if s_ch == sub_ch {
                    continue;
                }
                if s_ch.is_ascii() && sub_ch.is_ascii() && s_ch.eq_ignore_ascii_case(&sub_ch) {
                    continue;
                }
                if s_ch.is_ascii() && sub_ch.is_ascii() && s_ch.eq_ignore_ascii_case(&sub_ch) {
                    continue;
                }
                if s_ch.to_uppercase().eq(sub_ch.to_uppercase()) {
                    continue;
                }
                return false;
            }
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_roundtrip {
        ($key:expr, $text:expr) => {
            assert_eq!($key.to_string(), $text, "key to string");
            assert_eq!($text.parse::<Key>().unwrap(), $key, "string to key");
        };
    }

    macro_rules! assert_parse {
        ($key:expr, $text:expr) => {
            assert_eq!($text.parse::<Key>().unwrap(), $key, "string to key");
        };
    }

    macro_rules! assert_synonym {
        ($key:expr, $text:expr) => {
            assert_eq!($text.parse::<Key>().unwrap(), key($key));
        };
    }

    // https://users.rust-lang.org/t/ensure-exhaustiveness-of-list-of-enum-variants/99891/4
    macro_rules! exhaustive_list {
        ($E:path; $($variant:ident),* $(,)?) => {
            {
                use $E as E;
                let _ = |dummy: E| {
                    match dummy {
                        $(E::$variant => ()),*
                    }
                };
                [$(E::$variant),*]
            }
        }
    }

    #[test]
    fn key_db_covers_all_key_codes() {
        #[rustfmt::skip]
        let key_codes = exhaustive_list!(KeyCode;
            LAlt, LControl, LShift, LWin, RAlt, RControl, RShift, RWin,
            Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
            A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
            F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
            Numlock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
            NumpadAdd, NumpadDivide, NumpadDecimal, NumpadComma, NumpadEnter, NumpadEquals, NumpadMultiply, NumpadSubtract,
            AbntC1, AbntC2, Apostrophe, Apps, Asterisk, At, Ax, Backslash, Backspace, Calculator, Capital, Caret, Colon,
            Comma, Convert, Compose, Copy, Cut, Delete, Down, End, Enter, Equals, Escape, Grave, Home, Insert, Kana, Kanji,
            LBracket, Left, Mail, MediaSelect, MediaStop, Minus, Mute, MyComputer, NavigateForward, NavigateBackward,
            NextTrack, NoConvert, OEM102, PageDown, PageUp, Paste, Pause, Period, PlayPause, Plus, Power, PrevTrack,
            RBracket, Right, Scroll, Semicolon, Slash, Sleep, Snapshot, Space, Stop, Sysrq, Tab, Underline, Unlabeled,
            Up, VolumeDown, VolumeUp, Wake, WebBack, WebFavorites, WebForward, WebHome, WebRefresh, WebSearch, WebStop, Yen,
        );

        for key_code in key_codes {
            let _ = KEY_DB.key_code_symbol(key_code);
        }
    }

    #[test]
    fn numpad_canonicalized() {
        assert_eq!(KeyCode::Key1, key(KeyCode::Numpad1).1);
        assert_eq!(KeyCode::Key2, key(KeyCode::Numpad2).1);
        assert_eq!(KeyCode::Key3, key(KeyCode::Numpad3).1);
        assert_eq!(KeyCode::Key4, key(KeyCode::Numpad4).1);
        assert_eq!(KeyCode::Key5, key(KeyCode::Numpad5).1);
        assert_eq!(KeyCode::Key6, key(KeyCode::Numpad6).1);
        assert_eq!(KeyCode::Key7, key(KeyCode::Numpad7).1);
        assert_eq!(KeyCode::Key8, key(KeyCode::Numpad8).1);
        assert_eq!(KeyCode::Key9, key(KeyCode::Numpad9).1);
        assert_eq!(KeyCode::Key0, key(KeyCode::Numpad0).1);
        assert_eq!(KeyCode::Plus, key(KeyCode::NumpadAdd).1);
        assert_eq!(KeyCode::Slash, key(KeyCode::NumpadDivide).1);
        assert_eq!(KeyCode::Period, key(KeyCode::NumpadDecimal).1);
        assert_eq!(KeyCode::Comma, key(KeyCode::NumpadComma).1);
        assert_eq!(KeyCode::Enter, key(KeyCode::NumpadEnter).1);
        assert_eq!(KeyCode::Equals, key(KeyCode::NumpadEquals).1);
        assert_eq!(KeyCode::Asterisk, key(KeyCode::NumpadMultiply).1);
        assert_eq!(KeyCode::Minus, key(KeyCode::NumpadSubtract).1);
    }

    #[test]
    fn right_controls_canonical() {
        assert_eq!(KeyCode::LAlt, key(KeyCode::RAlt).1);
        assert_eq!(KeyCode::LControl, key(KeyCode::RControl).1);
        assert_eq!(KeyCode::LShift, key(KeyCode::RShift).1);
        assert_eq!(KeyCode::LWin, key(KeyCode::RWin).1);
    }

    #[test]
    fn roundtrip_simple() {
        assert_roundtrip!(key(KeyCode::Key0), "0");
        assert_roundtrip!(key(KeyCode::A), "A");
        assert_roundtrip!(key(KeyCode::F1), "F1");
        assert_roundtrip!(key(KeyCode::Apostrophe), "'");
        assert_roundtrip!(key(KeyCode::Caret), "^");

        assert_roundtrip!(key(KeyCode::LControl), CONTROL_SYMBOL);
        assert_roundtrip!(key(KeyCode::LAlt), ALT_SYMBOL);
        assert_roundtrip!(key(KeyCode::LShift), SHIFT_SYMBOL);
        assert_roundtrip!(key(KeyCode::LWin), LOGO_SYMBOL);
    }

    #[test]
    fn key_synonyms_parsing() {
        assert_synonym!(KeyCode::Apps, "Apps");
        assert_synonym!(KeyCode::Insert, "Ins");
        assert_synonym!(KeyCode::Insert, "Insert");

        assert_synonym!(KeyCode::LAlt, "Alt");
        assert_synonym!(KeyCode::LControl, "Ctrl");
        assert_synonym!(KeyCode::LControl, "Control");
        assert_synonym!(KeyCode::LShift, "Shift");
        assert_synonym!(KeyCode::LWin, "Command");
        assert_synonym!(KeyCode::LWin, "Cmd");
        assert_synonym!(KeyCode::LWin, "Logo");
        assert_synonym!(KeyCode::LWin, "Super");
        assert_synonym!(KeyCode::LWin, "Win");
    }

    #[test]
    fn case_insensitive_parsing() {
        assert_parse!(key(KeyCode::A), "a");
        assert_parse!(key(KeyCode::Insert), "insert");
        assert_parse!(key(KeyCode::Insert), "INSERT");
        assert_parse!(key(KeyCode::Insert), "Insert");
        assert_parse!(key(KeyCode::F1), "f1");
    }

    #[test]
    fn roundtrip_combinatory() {
        assert_roundtrip!(ctrl(key(KeyCode::A)), format!("{CONTROL_SYMBOL}A"));
        assert_roundtrip!(alt(key(KeyCode::A)), format!("{ALT_SYMBOL}A"));
        assert_roundtrip!(shift(key(KeyCode::A)), format!("{SHIFT_SYMBOL}A"));
        assert_roundtrip!(logo(key(KeyCode::A)), format!("{LOGO_SYMBOL}A"));

        assert_roundtrip!(
            ctrl(alt(shift(logo(key(KeyCode::A))))),
            format!("{CONTROL_SYMBOL}{ALT_SYMBOL}{SHIFT_SYMBOL}{LOGO_SYMBOL}A")
        );
    }

    #[test]
    fn combinatory_synonyms() {
        assert_eq!(shift(key(KeyCode::A)), "Shift-A".parse().unwrap());
        assert_eq!(ctrl(key(KeyCode::A)), "Ctrl-A".parse().unwrap());
        assert_eq!(ctrl(key(KeyCode::A)), "Control-A".parse().unwrap());
        assert_eq!(alt(key(KeyCode::A)), "Alt-A".parse().unwrap());
        assert_eq!(logo(key(KeyCode::A)), "Command-A".parse().unwrap());
        assert_eq!(logo(key(KeyCode::A)), "Cmd-A".parse().unwrap());

        assert_eq!(
            ctrl(alt(shift(logo(key(KeyCode::A))))),
            "Ctrl-Alt-Shift-Command-A".parse().unwrap()
        );
    }

    #[test]
    fn bad_key() {
        let result = "Garbage".parse::<Key>();
        assert!(
            result.is_err(),
            "Expected Err but got Ok({})",
            result.unwrap()
        );
        assert_eq!("invalid key value", result.err().unwrap().to_string());
    }
}
