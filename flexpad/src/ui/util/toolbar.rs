use super::{style::ToolbarStyle, TEXT_SIZE_TOOLTIP};
use crate::ui::util::action_tooltip;
use flexpad_toolkit::prelude::*;
use iced::{
    theme,
    widget::{button, container, horizontal_space, text, tooltip, vertical_rule, Row},
    Color, Element, Font, Length, Padding,
};

const TOOLBAR_HEIGHT: f32 = 25.0;
const TOOLBAR_BUTTON_WIDTH: f32 = 15.0;
const TOOLBAR_END_SPACE: f32 = SPACE_M;
const TOOLBAR_SEPARATOR_SIZE: f32 = 3.0;
const TOOLBAR_PADDING: Padding = Padding {
    top: SPACE_M,
    right: SPACE_S,
    bottom: SPACE_M,
    left: SPACE_S,
};
const TOOLBAR_ENABLED_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.80);
const TOOLBAR_DISABLED_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.50);

enum Entry<Message>
where
    Message: Clone,
{
    Action(Action, Option<Message>),
    Separator,
}

/// A toolbar that can display a set of actions in separated groups
pub struct Toolbar<Message>
where
    Message: Clone,
{
    entries: Vec<Entry<Message>>,
}

impl<Message> Toolbar<Message>
where
    Message: Clone,
{
    /// Create a new [`Toolbar`].
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Add a separator to the [`Toolbar`]
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// Add an action to the [`Toolbar`]
    pub fn action(mut self, action: impl Into<Action>, msg: Option<Message>) -> Self {
        self.entries.push(Entry::Action(action.into(), msg));
        self
    }
}

impl<Message> Default for Toolbar<Message>
where
    Message: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<Toolbar<Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Toolbar<Message>) -> Self {
        let mut buttons = Row::new()
            .height(TOOLBAR_HEIGHT)
            .push(horizontal_space(TOOLBAR_END_SPACE));
        for entry in value.entries {
            let element: Element<'a, Message> = match entry {
                Entry::Action(action, msg) => {
                    const ICON_FONT: Font = Font::with_name("flexpad-icons");

                    let color = match msg {
                        Some(_) => TOOLBAR_ENABLED_COLOR,
                        None => TOOLBAR_DISABLED_COLOR,
                    };

                    let codepoint = action
                        .icon_codepoint
                        .expect("Toolbar actions must have an icon codepoint");

                    let icon = text(codepoint)
                        .size(14)
                        .line_height(1.0)
                        .font(ICON_FONT)
                        .style(theme::Text::Color(color));

                    let content = container(icon)
                        .width(TOOLBAR_BUTTON_WIDTH)
                        .center_x()
                        .center_y();

                    let button = button(content).style(theme::Button::Text);

                    match msg {
                        Some(msg) => {
                            action_tooltip(&action, button.on_press(msg), tooltip::Position::Bottom)
                                .size(TEXT_SIZE_TOOLTIP)
                                .style(theme::Container::Box)
                                .into()
                        }
                        None => button.into(),
                    }
                }
                Entry::Separator => vertical_rule(TOOLBAR_SEPARATOR_SIZE)
                    .style(ToolbarStyle)
                    .into(),
            };
            buttons = buttons.push(element);
        }
        buttons = buttons.push(horizontal_space(TOOLBAR_END_SPACE));

        container(container(buttons).width(Length::Fill).style(ToolbarStyle))
            .width(Length::Fill)
            .padding(TOOLBAR_PADDING)
            .into()
    }
}
