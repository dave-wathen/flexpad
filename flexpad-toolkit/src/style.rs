use super::menu;
use iced::Color;

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
