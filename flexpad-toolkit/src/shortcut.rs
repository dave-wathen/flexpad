use crate::{action::Action, key::Key};
use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    event,
    mouse::{self, Cursor},
    overlay, Element, Event, Rectangle,
};

/// A container to trigger keyboard short-cut actions
pub struct Shortcut<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    action: Action,
    on_shortcut: Option<Message>,
}

impl<'a, Message, Renderer> Shortcut<'a, Message, Renderer> {
    /// Creates a [`Shortcut`] for the given [`Action`] with the given content.
    pub fn new(
        action: impl Into<Action>,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            action: action.into(),
            on_shortcut: None,
        }
    }

    /// Sets the message that will be produced when the shortcut of the [`Action`] is pressed
    ///
    /// Unless `on_shortcut` is called, the [`Shortcut`] will be not emit a message.
    pub fn on_shortcut(self, on_shortcut: Message) -> Self {
        self.on_shortcut_maybe(Some(on_shortcut))
    }

    /// Determines what will happen when the shortcut of the [`Action`] is pressed.
    /// If `on_shortcut` is Some then that message will be emitted.
    /// If `on_shortcut` is None then no message will be emitted.
    pub fn on_shortcut_maybe(self, on_shortcut: Option<Message>) -> Self {
        Self {
            on_shortcut,
            ..self
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Shortcut<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn width(&self) -> iced::Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> iced::Length {
        self.content.as_widget().height()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as renderer::Renderer>::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        } else if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key_code,
            modifiers,
        }) = event
        {
            if Some(Key::new(modifiers, key_code)) == self.action.shortcut {
                if self.on_shortcut.is_some() {
                    shell.publish(self.on_shortcut.as_ref().unwrap().clone());
                }
                return event::Status::Captured;
            }
        }

        event::Status::Ignored
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
    }
}

impl<'a, Message, Renderer> From<Shortcut<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(value: Shortcut<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}
