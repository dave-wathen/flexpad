use crate::{helpers::SPACE_M, menu};
use iced::{theme, widget, Color};

/// The style for a [`MenuBar`] and [`Menu`]s
#[derive(Debug, Clone, Copy, Default)]
pub enum MenuStyle {
    #[default]
    Normal,
}

impl menu::StyleSheet for iced::Theme {
    type Style = MenuStyle;

    fn menu_bar(&self, _style: &Self::Style) -> menu::MenuBarAppearance {
        let palette = self.extended_palette();
        menu::MenuBarAppearance {
            background: palette.background.weak.color.into(),
        }
    }

    fn active_menu_bar_item(&self, _style: &Self::Style) -> menu::MenuBarItemAppearance {
        let palette = self.extended_palette();
        menu::MenuBarItemAppearance {
            background: Color::TRANSPARENT.into(),
            text_color: palette.primary.base.text,
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn selected_menu_bar_item(&self, style: &Self::Style) -> menu::MenuBarItemAppearance {
        let palette = self.extended_palette();
        let active = self.active_menu_bar_item(style);
        menu::MenuBarItemAppearance {
            background: palette.background.strong.color.into(),
            text_color: palette.primary.base.text,
            ..active
        }
    }

    fn menu(&self, _style: &Self::Style) -> menu::MenuAppearance {
        let palette = self.extended_palette();
        menu::MenuAppearance {
            background: palette.background.weak.color.into(),
            separator_fill: palette.background.strong.color.into(),
            border_radius: 6.0,
            border_width: 1.0,
            border_color: palette.background.strong.color,
        }
    }

    fn inactive_action(&self, style: &Self::Style) -> menu::ActionAppearance {
        let active = self.active_action(style);
        let color = active.text_color;
        let faded = Color::from_rgba(color.r, color.g, color.b, color.a / 2.0);
        menu::ActionAppearance {
            text_color: faded,
            shortcut_color: faded,
            ..active
        }
    }

    fn active_action(&self, _style: &Self::Style) -> menu::ActionAppearance {
        let palette = self.extended_palette();
        let color = palette.primary.base.text;
        let faded = Color::from_rgba(color.r, color.g, color.b, color.a / 2.0);
        menu::ActionAppearance {
            background: Color::TRANSPARENT.into(),
            text_color: color,
            shortcut_color: faded,
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused_selected_action(&self, style: &Self::Style) -> menu::ActionAppearance {
        let palette = self.extended_palette();
        let color = palette.primary.strong.text;
        let faded = Color::from_rgba(color.r, color.g, color.b, color.a / 2.0);

        let active = self.active_action(style);
        menu::ActionAppearance {
            background: palette.primary.strong.color.into(),
            text_color: color,
            shortcut_color: faded,
            ..active
        }
    }

    fn unfocused_selected_action(&self, style: &Self::Style) -> menu::ActionAppearance {
        let palette = self.extended_palette();
        let active = self.active_action(style);
        menu::ActionAppearance {
            background: palette.background.strong.color.into(),
            ..active
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

impl super::dialog::StyleSheet for iced::Theme {
    type Style = DialogStyle;

    fn active(&self, style: &Self::Style) -> super::dialog::Appearance {
        let palette = self.extended_palette();

        match style {
            DialogStyle::Normal => super::dialog::Appearance {
                border_color: palette.primary.base.color,
                border_radius: SPACE_M,
                background: palette.background.base.color.into(),
                title_background: palette.primary.base.color.into(),
            },
            DialogStyle::Error => super::dialog::Appearance {
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

/// A style that can be applied to an action
#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    /// Normal stylimng for an Ok action
    Ok,
    /// Normal stylimng for a Cancel action
    Cancel,
    /// Styling for when an action (should be Ok) is used to apply to a error situation
    Error,
}

impl widget::button::StyleSheet for ButtonStyle
where
    iced::Theme: widget::button::StyleSheet<Style = theme::Button>,
{
    type Style = theme::Theme;

    fn active(&self, theme: &Self::Style) -> widget::button::Appearance {
        match self {
            Self::Ok => theme.active(&theme::Button::Primary),
            Self::Cancel => theme.active(&theme::Button::Secondary),
            Self::Error => theme.active(&theme::Button::Destructive),
        }
    }
}

impl From<ButtonStyle> for theme::Button {
    fn from(value: ButtonStyle) -> Self {
        theme::Button::Custom(Box::new(value))
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
        let palette = iced::Theme::Light.extended_palette();
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
