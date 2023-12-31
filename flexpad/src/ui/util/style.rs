use flexpad_toolkit::prelude::*;
use iced::{
    theme,
    widget::{self, button, rule::FillMode},
    Color, Theme,
};

pub struct ToolbarStyle;

impl From<ToolbarStyle> for theme::Container {
    fn from(value: ToolbarStyle) -> Self {
        theme::Container::Custom(Box::new(value))
    }
}

impl From<ToolbarStyle> for theme::Rule {
    fn from(value: ToolbarStyle) -> Self {
        theme::Rule::Custom(Box::new(value))
    }
}

impl widget::container::StyleSheet for ToolbarStyle {
    type Style = iced::Theme;

    fn appearance(&self, theme: &Self::Style) -> widget::container::Appearance {
        let palette = theme.extended_palette();

        widget::container::Appearance {
            text_color: None,
            background: Some(palette.secondary.base.color.into()),
            border_radius: SPACE_M.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl widget::rule::StyleSheet for ToolbarStyle {
    type Style = iced::Theme;

    fn appearance(&self, theme: &Self::Style) -> widget::rule::Appearance {
        let dflt = theme.appearance(&iced::theme::Rule::Default);

        widget::rule::Appearance {
            fill_mode: FillMode::Padded(4),
            ..dflt
        }
    }
}

pub enum TextStyle {
    Default,
    Label,
    Error,
    Loading,
}

impl From<TextStyle> for theme::Text {
    fn from(value: TextStyle) -> Self {
        // TODO Theme - there's no Custom for theme::Text!
        let palette = Theme::Light.extended_palette();
        let color = match value {
            TextStyle::Default => palette.primary.base.text,
            TextStyle::Label => palette.primary.weak.text,
            TextStyle::Error => palette.danger.base.color,
            TextStyle::Loading => palette.secondary.strong.color,
        };
        theme::Text::Color(color)
    }
}

pub enum TextInputStyle {
    Default,
    Error,
}

impl From<TextInputStyle> for theme::TextInput {
    fn from(value: TextInputStyle) -> Self {
        theme::TextInput::Custom(Box::new(value))
    }
}

impl widget::text_input::StyleSheet for TextInputStyle
where
    iced::Theme: widget::text_input::StyleSheet<Style = theme::TextInput>,
{
    type Style = iced::Theme;

    fn active(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.active(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.strong.color,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn focused(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.focused(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.strong.color,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn disabled(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.disabled(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.weak.color,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn placeholder_color(&self, theme: &Self::Style) -> iced::Color {
        theme.placeholder_color(&theme::TextInput::Default)
    }

    fn value_color(&self, theme: &Self::Style) -> iced::Color {
        theme.value_color(&theme::TextInput::Default)
    }

    fn disabled_color(&self, theme: &Self::Style) -> iced::Color {
        theme.disabled_color(&theme::TextInput::Default)
    }

    fn selection_color(&self, theme: &Self::Style) -> iced::Color {
        theme.selection_color(&theme::TextInput::Default)
    }
}

/// A style that can be applied to an action
#[derive(Debug, Clone, Copy)]
pub enum DialogButtonStyle {
    /// Normal stylimng for an Ok action
    Ok,
    /// Normal stylimng for a Cancel action
    Cancel,
    /// Styling for when an action (should be Ok) is used to apply to a error situation
    Error,
}

impl widget::button::StyleSheet for DialogButtonStyle
where
    iced::Theme: widget::button::StyleSheet<Style = theme::Button>,
{
    type Style = theme::Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        match self {
            Self::Ok => theme.active(&theme::Button::Primary),
            Self::Cancel => theme.active(&theme::Button::Secondary),
            Self::Error => theme.active(&theme::Button::Destructive),
        }
    }
}

/// The style for a [`Dialog`]
#[derive(Debug, Clone, Copy, Default)]
pub enum DialogStyle {
    #[default]
    Normal,
    Error,
}

impl crate::ui::util::dialog::StyleSheet for iced::Theme {
    type Style = DialogStyle;

    fn active(&self, style: &Self::Style) -> crate::ui::util::dialog::Appearance {
        let palette = self.extended_palette();

        match style {
            DialogStyle::Normal => crate::ui::util::dialog::Appearance {
                border_color: palette.primary.base.color,
                border_radius: SPACE_M,
                background: palette.background.base.color.into(),
                title_background: palette.primary.base.color.into(),
            },
            DialogStyle::Error => crate::ui::util::dialog::Appearance {
                border_color: palette.danger.base.color,
                border_radius: SPACE_M,
                background: palette.background.base.color.into(),
                title_background: palette.danger.base.color.into(),
            },
        }
    }
}

impl From<DialogStyle> for theme::Text {
    fn from(value: DialogStyle) -> Self {
        match value {
            DialogStyle::Normal => theme::Text::Color(Color::WHITE),
            DialogStyle::Error => theme::Text::Color(Color::WHITE),
        }
    }
}
