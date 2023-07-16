use std::rc::Rc;

use iced::{Background, Color, Theme};

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the grid.
    pub background: Option<Background>,
    /// The width of rule lines for the grid.
    pub rule_width: f32,
    /// The [`Color`] of rule lines for the grid.
    pub rule_color: Color,
    /// The width of rule lines for the grid.
    pub heads_rule_width: f32,
    /// The [`Color`] of rule lines for the grid.
    pub heads_rule_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            background: None,
            rule_width: 0.0,
            rule_color: Color::TRANSPARENT,
            heads_rule_width: 0.0,
            heads_rule_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a grid.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a grid.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

/// The style of a grid.
#[derive(Default, Clone)]
pub enum Grid {
    /// No style.
    #[default]
    Plain,
    /// A grid with rule lines.
    Ruled,
    /// A custom style.
    Custom(Rc<dyn StyleSheet<Style = Theme>>),
}

impl StyleSheet for Theme {
    type Style = Grid;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            Grid::Plain => Default::default(),
            Grid::Ruled => {
                let palette = self.extended_palette();

                Appearance {
                    background: Some(palette.background.base.color.into()),
                    rule_color: Color {
                        a: palette.secondary.base.color.a * 0.4,
                        ..palette.secondary.base.color
                    },
                    rule_width: 1.0,
                    heads_rule_color: palette.secondary.base.color,
                    heads_rule_width: 1.0,
                }
            }
            Grid::Custom(custom) => custom.appearance(self),
        }
    }
}
