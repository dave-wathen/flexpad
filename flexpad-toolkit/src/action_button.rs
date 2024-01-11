use crate::{action::Action, helpers::shortcut};
use iced::{
    alignment,
    widget::{button, text},
    Element,
};

/// A button that displays an [`Action`] (short_name) and will trigger if the action's shortcut key is used.
pub struct ActionButton<Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: button::StyleSheet,
{
    action: Action,
    on_press: Option<Message>,
    style: <Renderer::Theme as button::StyleSheet>::Style,
}

impl<Message, Renderer> ActionButton<Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: button::StyleSheet,
{
    /// Create a new ActionButton
    pub fn new(action: impl Into<Action>) -> Self {
        Self {
            action: action.into(),
            on_press: None,
            style: <Renderer::Theme as button::StyleSheet>::Style::default(),
        }
    }

    /// Sets the message that will be produced when the [`ActionButton`] is pressed or the action's
    /// keyboard shortcut is pressed
    ///
    /// Unless `on_press` is called, the [`ActionButton`] will be disabled and the action's shortcut
    /// will not emit a message.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Sets the style variant of this [`Button`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as button::StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> From<ActionButton<Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet,
    Renderer::Theme: button::StyleSheet,
{
    fn from(value: ActionButton<Message, Renderer>) -> Self {
        shortcut(
            value.action.clone(),
            button(
                text(value.action.short_name).horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(100.0) // TODO look at new layout functionality to solve in a more adaptable fashion!
            .style(value.style)
            .on_press_maybe(value.on_press.clone()),
        )
        .on_shortcut_maybe(value.on_press)
        .into()
    }
}
