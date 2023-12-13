use iced::{
    advanced::{
        layout::{self, Node},
        mouse, overlay, renderer,
        widget::{self, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    event, Background, Color, Element, Event, Length, Point, Rectangle, Size,
};

use crate::ui::util::SPACE_M;

/// A Dialog (to be used with modal)
pub struct Dialog<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    title: Element<'a, Message, Renderer>,
    body: Element<'a, Message, Renderer>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Dialog<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Create a new [`Dialog`] with the given title and body elements
    pub fn new(
        title: impl Into<Element<'a, Message, Renderer>>,
        body: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Shrink,
            max_width: u32::MAX as f32,
            max_height: u32::MAX as f32,
            title: title.into(),
            body: body.into(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Dialog`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Dialog`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum height of the [`Dialog`].
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Sets the maximum width of the [`Dialog`].
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    /// Sets the style of the [`Dialog`].
    pub fn style(mut self, style: <Renderer::Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Dialog<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn width(&self) -> iced::Length {
        self.width
    }

    fn height(&self) -> iced::Length {
        self.height
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.title), Tree::new(&self.body)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.title, &self.body]);
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width).max_height(self.max_height);

        let padding = SPACE_M.into();
        let limits = limits
            .loose()
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut title = self
            .title
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits);
        title.move_to(Point::new(SPACE_M, SPACE_M));
        let limits = limits.shrink(Size::new(0.0, title.size().height + SPACE_M * 2.0));

        let mut body = self
            .body
            .as_widget()
            .layout(&mut tree.children[1], renderer, &limits);
        body.move_to(Point::new(
            SPACE_M,
            SPACE_M + title.size().height + SPACE_M * 2.0,
        ));

        let content_size = Size::new(
            body.size().width.max(title.size().width),
            title.size().height + SPACE_M * 2.0 + body.size().height,
        );

        let size = limits.resolve(content_size);
        Node::with_children(size.pad(padding), vec![title, body])
    }

    fn draw(
        &self,
        state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let appearance = theme.active(&self.style);
        let bounds = layout.bounds();

        let mut children = layout.children();

        // Background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: appearance.border_radius.into(),
                border_width: 1.0,
                border_color: appearance.border_color,
            },
            appearance.background,
        );

        let title_layout = children.next().expect("Title layout expected");
        // Title Background - Step 1
        let title_height = title_layout.bounds().height + SPACE_M * 2.0;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: title_height,
                },
                border_radius: appearance.border_radius.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            appearance.title_background,
        );
        // Title Background - Step 2 (Remove lower round corners)
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + title_height - appearance.border_radius,
                    width: bounds.width,
                    height: appearance.border_radius,
                },
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            appearance.title_background,
        );
        self.title.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            title_layout,
            cursor,
            viewport,
        );

        let body_layout = children.next().expect("Body layout expected");
        self.body.as_widget().draw(
            &state.children[1],
            renderer,
            theme,
            style,
            body_layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            [&self.title, &self.body]
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
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
        [&mut self.title, &mut self.body]
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        [&self.title, &self.body]
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let mut child_layouts = layout.children();
        let mut child_states = tree.children.iter_mut();

        let (title, body) = (
            self.title.as_widget_mut().overlay(
                child_states.next().expect("Expected title state"),
                child_layouts.next().expect("Expected title layout"),
                renderer,
            ),
            self.body.as_widget_mut().overlay(
                child_states.next().expect("Expected body state"),
                child_layouts.next().expect("Expected body layout"),
                renderer,
            ),
        );

        match (title, body) {
            (None, None) => None,
            (None, Some(body)) => Some(body),
            (Some(title), None) => Some(title),
            (Some(title), Some(body)) => {
                Some(overlay::Group::with_children(vec![title, body]).into())
            }
        }
    }
}

impl<'a, Message, Renderer> From<Dialog<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(dialog: Dialog<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(dialog)
    }
}

/// StyleSheet to deterrmine the appearance of a [`Dialog`]
pub trait StyleSheet {
    type Style: Default;
    /// The normal appearance of a [`Dialog`]
    fn active(&self, style: &Self::Style) -> Appearance;
}

/// The appearance of a [`Dialog`].
#[derive(Clone, Copy, Debug)]
pub struct Appearance {
    /// The border color of the [`Dialog`]
    pub border_color: Color,

    /// The border radius of the [`Dialog`]
    pub border_radius: f32,

    /// The background of the [`Dialog`]
    pub background: Background,

    /// The background of the title of the [`Dialog`]
    pub title_background: Background,
}
