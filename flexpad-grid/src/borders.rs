use iced::Color;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Border {
    /// The width of the [Border].
    pub width: f32,
    /// The [`Color`] of the [Border].
    pub color: Color,
}

#[allow(dead_code)]
impl Border {
    pub const NONE: Border = Border {
        width: 0.0,
        color: Color::TRANSPARENT,
    };

    pub fn new(width: f32, color: impl Into<Color>) -> Self {
        Self {
            width,
            color: color.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Borders {
    // The top [`Border`]
    pub top: Border,
    // The right [`Border`]
    pub right: Border,
    // The bottom [`Border`]
    pub bottom: Border,
    // The left [`Border`]
    pub left: Border,
}

#[allow(dead_code)]
impl Borders {
    pub const NONE: Borders = Borders {
        top: Border::NONE,
        right: Border::NONE,
        bottom: Border::NONE,
        left: Border::NONE,
    };

    // Create a [`Borders`] that is the same on all sides
    pub fn new(border: impl Into<Border>) -> Self {
        let border = border.into();
        Self {
            top: border,
            right: border,
            bottom: border,
            left: border,
        }
    }

    // Create a [`Borders`] that only sets a top [`Border`]
    pub fn top(border: impl Into<Border>) -> Self {
        Self {
            top: border.into(),
            ..Borders::NONE
        }
    }

    // Create a [`Borders`] that only sets a right [`Border`]
    pub fn right(border: impl Into<Border>) -> Self {
        Self {
            right: border.into(),
            ..Borders::NONE
        }
    }

    // Create a [`Borders`] that only sets a bottom [`Border`]
    pub fn bottom(border: impl Into<Border>) -> Self {
        Self {
            bottom: border.into(),
            ..Borders::NONE
        }
    }

    // Create a [`Borders`] that only sets a left [`Border`]
    pub fn left(border: impl Into<Border>) -> Self {
        Self {
            left: border.into(),
            ..Borders::NONE
        }
    }

    pub fn overlay(&self, other: &Borders) -> Self {
        let choose = |s, o| if o == Border::NONE { s } else { o };

        Self {
            top: choose(self.top, other.top),
            right: choose(self.right, other.right),
            bottom: choose(self.bottom, other.bottom),
            left: choose(self.left, other.left),
        }
    }
}

#[cfg(test)]
mod test {
    use iced::color;

    use super::*;

    #[test]
    fn overlay() {
        let b1: Border = Border::new(1.0, color!(255, 0, 0));
        let b2: Border = Border::new(2.0, color!(0, 255, 0));
        let b3: Border = Border::new(3.0, color!(0, 0, 255));
        let b4: Border = Border::new(4.0, color!(255, 255, 255));

        let borders = Borders::top(b1).overlay(&Borders::bottom(b2));
        assert_eq!(b1, borders.top);
        assert_eq!(Border::NONE, borders.right);
        assert_eq!(b2, borders.bottom);
        assert_eq!(Border::NONE, borders.left);

        let borders = Borders::left(b1).overlay(&Borders::left(b2));
        assert_eq!(Border::NONE, borders.top);
        assert_eq!(Border::NONE, borders.right);
        assert_eq!(Border::NONE, borders.bottom);
        assert_eq!(b2, borders.left);

        let borders = Borders::top(b1)
            .overlay(&Borders::right(b2))
            .overlay(&Borders::bottom(b3))
            .overlay(&Borders::left(b4));
        assert_eq!(b1, borders.top);
        assert_eq!(b2, borders.right);
        assert_eq!(b3, borders.bottom);
        assert_eq!(b4, borders.left);

        let borders2 = Borders::top(b4)
            .overlay(&Borders::right(b3))
            .overlay(&Borders::bottom(b2))
            .overlay(&Borders::left(b1));
        let borders = borders.overlay(&borders2);
        assert_eq!(b4, borders.top);
        assert_eq!(b3, borders.right);
        assert_eq!(b2, borders.bottom);
        assert_eq!(b1, borders.left);
    }
}
