//! Navigate an endless amount of content with a scrollbar.
use iced::advanced::widget::{self, tree, Operation, Tree};
use iced::advanced::{layout, renderer, Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::overlay;
use iced::touch;
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset, StyleSheet};
use iced::{keyboard, window};
use iced::{mouse, Command};
use iced::{Color, Element, Length, Pixels, Rectangle, Size, Vector};

use crate::{CellRange, Grid, RowCol};

mod operation;
mod state;

use state::{Granularity, GridScrollableState};

/// A widget that can display a large [`Grid`] with scrollbars
#[allow(missing_debug_implementations)]
pub struct GridScrollable<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    vertical: Properties,
    horizontal: Properties,
    vertical_granularity: Granularity,
    horizontal_granularity: Granularity,
    content: Grid<'a, Message, Renderer>,
    on_viewport_change: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> GridScrollable<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
{
    /// Creates a new [`GridScrollable`].
    pub fn new(content: Grid<'a, Message, Renderer>) -> Self {
        GridScrollable {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            vertical: Default::default(),
            horizontal: Default::default(),
            vertical_granularity: Default::default(),
            horizontal_granularity: Default::default(),
            content,
            on_viewport_change: None,
            style: Default::default(),
        }
    }

    /// Sets the [`Id`] of the [`GridScrollable`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the width of the [`GridScrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`GridScrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Configures the vertical scrollbar of the [`GridScrollable`] .
    pub fn vertical_scroll(mut self, properties: Properties) -> Self {
        self.vertical = properties;
        self
    }

    /// Configures the horizontal scrollbar of the [`GridScrollable`] .
    pub fn horizontal_scroll(mut self, properties: Properties) -> Self {
        self.horizontal = properties;
        self
    }

    /// Configures the vertical scrolling behavour of the [`GridScrollable`] .
    pub fn vertical_scroll_smoothly(mut self) -> Self {
        self.vertical_granularity = Granularity::Continuous;
        self
    }

    /// Configures the vertical scrolling behavour of the [`GridScrollable`] .
    pub fn vertical_scroll_by_row(mut self) -> Self {
        self.vertical_granularity = Granularity::Discrete;
        self
    }

    /// Configures the horizontal scrolling behavour of the [`GridScrollable`] .
    pub fn horizontal_scroll_smoothly(mut self) -> Self {
        self.horizontal_granularity = Granularity::Continuous;
        self
    }

    /// Configures the horizontal scrolling behavour of the [`GridScrollable`] .
    pub fn horizontal_scroll_by_column(mut self) -> Self {
        self.horizontal_granularity = Granularity::Discrete;
        self
    }

    /// Configures the scrolling behavour of the [`GridScrollable`] for both dimensions.
    pub fn scroll_smoothly(mut self) -> Self {
        self.vertical_granularity = Granularity::Continuous;
        self.horizontal_granularity = Granularity::Continuous;
        self
    }

    /// Configures the scrolling behavour of the [`GridScrollable`] for both dimensions.
    pub fn scroll_by_row_and_column(mut self) -> Self {
        self.vertical_granularity = Granularity::Discrete;
        self.horizontal_granularity = Granularity::Discrete;
        self
    }

    /// Sets a function to call when the [`GridScrollable`] changes its viewport onto the
    /// scrolled content.  This happens when the [`GridScrollable`] is first painted,
    /// scrolled or resized.
    ///
    /// The function takes the [`Viewport`] of the [`GridScrollable`]
    pub fn on_viewport_change(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_viewport_change = Some(Box::new(f));
        self
    }

    /// Sets the style of the [`GridScrollable`] .
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer> for GridScrollable<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GridScrollableState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(GridScrollableState::new(
            self.horizontal_granularity,
            self.vertical_granularity,
        ))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() == 1 {
            tree.children[0].diff(&self.content);
        } else {
            tree.children = vec![Tree::new(&self.content)];
        }
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let child_limits = layout::Limits::new(
            Size::new(limits.min().width, 0.0),
            Size::new(f32::INFINITY, f32::MAX),
        );

        let content = self.content.layout(renderer, &child_limits);
        let size = limits.resolve(content.size());

        // Add buffer space to ensure last row/column can be seen
        let Size { width, height } = size;
        let (x_buffer, y_buffer) = match (self.horizontal_granularity, self.vertical_granularity) {
            (Granularity::Discrete, Granularity::Discrete) => (width * 0.75, height * 0.75),
            (Granularity::Discrete, Granularity::Continuous) => (width * 0.75, 0.0),
            (Granularity::Continuous, Granularity::Discrete) => (0.0, height * 0.75),
            (Granularity::Continuous, Granularity::Continuous) => (0.0, 0.0),
        };
        let content_bounds = content.bounds();
        let content_children: Vec<layout::Node> = content.children().to_vec();
        let content_size = Size::new(
            content_bounds.size().width + x_buffer,
            content_bounds.size().height + y_buffer,
        );
        let mut content = layout::Node::with_children(content_size, content_children);
        content.move_to(content_bounds.position());

        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<GridScrollableState>();
        state.calculate_parts_and_update(
            layout.bounds(),
            self.horizontal,
            self.vertical,
            &tree.children[0],
            layout.children().next().expect("Grid layout missing"),
        );

        operation.custom(state, self.id.as_ref().map(|id| &id.0));

        let translation = state.absolute_offset();
        operation.scrollable(
            state,
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            translation,
        );

        operation.container(
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            &mut |operation| {
                self.content.operate(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            },
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
        let state = tree.state.downcast_mut::<GridScrollableState>();
        let parts = state.calculate_parts_and_update(
            layout.bounds(),
            self.horizontal,
            self.vertical,
            &tree.children[0],
            layout.children().next().expect("Grid layout missing"),
        );

        let cursor_over_scrollable = cursor.position_over(parts.full_bounds());

        let content_layout = layout.children().next().unwrap();

        let over_x_scrollbar = parts.is_mouse_over_x_scrollbar(cursor);
        let over_y_scrollbar = parts.is_mouse_over_y_scrollbar(cursor);

        let event_status = {
            let cursor = match cursor_over_scrollable {
                Some(cursor_position) if !(over_x_scrollbar || over_y_scrollbar) => {
                    mouse::Cursor::Available(cursor_position + state.absolute_offset())
                }
                _ => mouse::Cursor::Unavailable,
            };

            self.content.on_event(
                &mut tree.children[0],
                event.clone(),
                content_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        };

        if let event::Status::Captured = event_status {
            return event::Status::Captured;
        }

        if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.keyboard_modifiers = modifiers;

            return event::Status::Ignored;
        }

        match event {
            Event::Window(window::Event::Resized { width, height }) => {
                let new_bounds = Rectangle::new(
                    layout.bounds().position(),
                    Size::new(width as f32, height as f32),
                );
                state.calculate_parts_and_update(
                    new_bounds,
                    self.horizontal,
                    self.vertical,
                    &tree.children[0],
                    layout.children().next().expect("Grid layout missing"),
                );
                state.notify_viewport_change(
                    &self.on_viewport_change,
                    parts.cells_viewport.size(),
                    shell,
                );
                return event::Status::Ignored;
            }
            Event::Window(window::Event::RedrawRequested(_)) if !state.is_viewport_notified() => {
                state.notify_viewport_change(
                    &self.on_viewport_change,
                    parts.cells_viewport.size(),
                    shell,
                );
                return event::Status::Ignored;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor_over_scrollable.is_none() {
                    return event::Status::Ignored;
                }

                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // TODO: Configurable speed/friction (?)
                        let movement = if state.keyboard_modifiers.shift() {
                            Vector::new(y, x)
                        } else {
                            Vector::new(x, y)
                        };

                        movement * 60.0
                    }
                    mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
                };

                state.scroll(delta, parts.full_bounds());

                state.notify_viewport_change(
                    &self.on_viewport_change,
                    parts.cells_viewport.size(),
                    shell,
                );
                return event::Status::Captured;
            }
            Event::Touch(event)
                if state.scroll_area_touched_at.is_some()
                    || !over_y_scrollbar && !over_x_scrollbar =>
            {
                match event {
                    touch::Event::FingerPressed { .. } => {
                        let Some(cursor_position) = cursor.position() else {
                            return event::Status::Ignored;
                        };

                        state.scroll_area_touched_at = Some(cursor_position);
                    }
                    touch::Event::FingerMoved { .. } => {
                        if let Some(scroll_box_touched_at) = state.scroll_area_touched_at {
                            let Some(cursor_position) = cursor.position() else {
                                return event::Status::Ignored;
                            };

                            let delta = Vector::new(
                                cursor_position.x - scroll_box_touched_at.x,
                                cursor_position.y - scroll_box_touched_at.y,
                            );

                            state.scroll(delta, parts.cells_viewport);
                            state.scroll_area_touched_at = Some(cursor_position);
                            state.notify_viewport_change(
                                &self.on_viewport_change,
                                parts.cells_viewport.size(),
                                shell,
                            );
                        }
                    }
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. } => {
                        state.scroll_area_touched_at = None;
                    }
                }

                return event::Status::Captured;
            }
            _ => {}
        }

        if let Some(scroller_grabbed_at) = state.y_scroller_grabbed_at {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    state.y_scroller_grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if let Some(scrollbar) = parts.y_scrollbar {
                        let Some(cursor_position) = cursor.position() else {
                            return event::Status::Ignored;
                        };

                        state.scroll_y_to(
                            scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                            parts.cells_viewport.height,
                        );
                        state.notify_viewport_change(
                            &self.on_viewport_change,
                            parts.cells_viewport.size(),
                            shell,
                        );

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        } else if over_y_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored;
                    };

                    if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                        (parts.grab_y_scroller(cursor_position), parts.y_scrollbar)
                    {
                        state.scroll_y_to(
                            scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                            parts.cells_viewport.height,
                        );
                        state.y_scroller_grabbed_at = Some(scroller_grabbed_at);
                        state.notify_viewport_change(
                            &self.on_viewport_change,
                            parts.cells_viewport.size(),
                            shell,
                        );
                    }

                    return event::Status::Captured;
                }
                _ => {}
            }
        }

        if let Some(scroller_grabbed_at) = state.x_scroller_grabbed_at {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    state.x_scroller_grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored;
                    };

                    if let Some(scrollbar) = parts.x_scrollbar {
                        state.scroll_x_to(
                            scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                            parts.cells_viewport.height,
                        );
                        state.notify_viewport_change(
                            &self.on_viewport_change,
                            parts.cells_viewport.size(),
                            shell,
                        );
                    }

                    return event::Status::Captured;
                }
                _ => {}
            }
        } else if over_x_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored;
                    };

                    if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                        (parts.grab_x_scroller(cursor_position), parts.x_scrollbar)
                    {
                        state.scroll_x_to(
                            scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                            parts.cells_viewport.width,
                        );
                        state.x_scroller_grabbed_at = Some(scroller_grabbed_at);
                        state.notify_viewport_change(
                            &self.on_viewport_change,
                            parts.cells_viewport.size(),
                            shell,
                        );

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<GridScrollableState>();
        let parts = state.calculate_parts(
            layout.bounds(),
            self.horizontal,
            self.vertical,
            &tree.children[0],
            layout.children().next().expect("Grid layout missing"),
        );

        let content_layout = layout.children().next().unwrap();

        let cursor_over_scrollable = cursor.position_over(parts.full_bounds());
        let over_x_scrollbar = parts.is_mouse_over_x_scrollbar(cursor);
        let over_y_scrollbar = parts.is_mouse_over_y_scrollbar(cursor);

        let offset = state.absolute_offset();

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(over_x_scrollbar || over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + offset)
            }
            _ => mouse::Cursor::Unavailable,
        };

        if parts.can_scroll() {
            // Draw grid content
            renderer.with_layer(parts.cells_viewport, |renderer| {
                renderer.with_translation(Vector::ZERO - offset, |renderer| {
                    self.content.draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        content_layout,
                        cursor,
                        &Rectangle::new(
                            parts.cells_viewport.position() + offset,
                            parts.cells_viewport.size(),
                        ),
                    );
                });
            });

            // Draw row heads
            if let Some(rh_viewport) = parts.row_heads_viewport {
                renderer.with_layer(rh_viewport, |renderer| {
                    let head_offset = Vector::new(0.0, offset.y);
                    renderer.with_translation(Vector::ZERO - head_offset, |renderer| {
                        self.content.draw(
                            &tree.children[0],
                            renderer,
                            theme,
                            style,
                            content_layout,
                            cursor,
                            &Rectangle::new(
                                rh_viewport.position() + head_offset,
                                rh_viewport.size(),
                            ),
                        );
                    });
                });
            }

            // Draw column heads
            if let Some(ch_viewport) = parts.column_heads_viewport {
                renderer.with_layer(ch_viewport, |renderer| {
                    let head_offset = Vector::new(offset.x, 0.0);
                    renderer.with_translation(Vector::ZERO - head_offset, |renderer| {
                        self.content.draw(
                            &tree.children[0],
                            renderer,
                            theme,
                            style,
                            content_layout,
                            cursor,
                            &Rectangle::new(
                                ch_viewport.position() + head_offset,
                                ch_viewport.size(),
                            ),
                        );
                    });
                });
            }

            // Draw corner
            if let Some(c_viewport) = parts.corner_viewport {
                renderer.with_layer(c_viewport, |renderer| {
                    self.content.draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        content_layout,
                        cursor,
                        &c_viewport,
                    );
                });
            }

            renderer.with_layer(
                Rectangle {
                    width: parts.full_bounds().width + 2.0,
                    height: parts.full_bounds().height + 2.0,
                    ..parts.full_bounds()
                },
                |renderer| {
                    //draw y scrollbar
                    if let Some(scrollbar) = parts.y_scrollbar {
                        let style = if state.is_y_scroller_grabbed() {
                            theme.dragging(&self.style)
                        } else if cursor_over_scrollable.is_some() {
                            theme.hovered(&self.style, over_y_scrollbar)
                        } else {
                            theme.active(&self.style)
                        };

                        scrollbar.draw(renderer, style);
                    }

                    //draw x scrollbar
                    if let Some(scrollbar) = parts.x_scrollbar {
                        let style = if state.is_x_scroller_grabbed() {
                            theme.dragging_horizontal(&self.style)
                        } else if cursor_over_scrollable.is_some() {
                            theme.hovered_horizontal(&self.style, over_x_scrollbar)
                        } else {
                            theme.active_horizontal(&self.style)
                        };

                        scrollbar.draw(renderer, style);
                    }

                    // TODO: make fill in color configurable
                    let active_style = theme.active(&self.style);
                    let color = active_style.scroller.color;
                    let color = Color::new(
                        (color.r * 1.1).min(1.0),
                        (color.g * 1.1).min(1.0),
                        (color.b * 1.1).min(1.0),
                        color.a,
                    );
                    for fill_in in parts.fill_ins() {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: fill_in,
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            color,
                        );
                    }
                },
            );
        } else {
            self.content.draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                &Rectangle {
                    x: parts.full_bounds().x + offset.x,
                    y: parts.full_bounds().y + offset.y,
                    ..parts.full_bounds()
                },
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<GridScrollableState>();
        let parts = state.calculate_parts(
            layout.bounds(),
            self.horizontal,
            self.vertical,
            &tree.children[0],
            layout.children().next().expect("Grid layout missing"),
        );
        let cursor_over_scrollable = cursor.position_over(parts.full_bounds());

        let content_layout = layout.children().next().unwrap();

        let over_x_scrollbar = parts.is_mouse_over_x_scrollbar(cursor);
        let over_y_scrollbar = parts.is_mouse_over_y_scrollbar(cursor);

        if (over_x_scrollbar || over_y_scrollbar) || state.is_a_scroller_grabbed() {
            mouse::Interaction::Idle
        } else {
            let offset = state.absolute_offset();

            let cursor = match cursor_over_scrollable {
                Some(cursor_position) if !(over_x_scrollbar || over_y_scrollbar) => {
                    mouse::Cursor::Available(cursor_position + offset)
                }
                _ => mouse::Cursor::Unavailable,
            };

            self.content.mouse_interaction(
                &tree.children[0],
                content_layout,
                cursor,
                &Rectangle {
                    y: parts.full_bounds().y + offset.y,
                    x: parts.full_bounds().x + offset.x,
                    ..parts.full_bounds()
                },
                renderer,
            )
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let offset = tree
            .state
            .downcast_ref::<GridScrollableState>()
            .absolute_offset();
        self.content
            .overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
            .map(|overlay| overlay.translate(Vector::new(-offset.x, -offset.y)))
    }
}

/// Produces a [`Command`] that scrolls the [`GridScrollable`] with the given [`Id`]
/// to ensure that a cell is available.
pub fn ensure_cell_visible(id: Id, cell: RowCol) -> Command<Viewport> {
    Command::widget(operation::ensure_cell_visible(id.0, cell))
}

/// Produces a [`Command`] that returns the viewport of the  [`GridScrollable`] with the given [`Id`].
pub fn get_viewport(id: Id) -> Command<Viewport> {
    Command::widget(operation::get_viewport(id.0))
}

impl<'a, Message, Renderer> From<GridScrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
{
    fn from(text_input: GridScrollable<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The identifier of a [`GridScrollable`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// The current [`Viewport`] of the [`GridScrollable`].
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    absolute: AbsoluteOffset,
    relative: RelativeOffset,
    range: CellRange,
}

impl Viewport {
    pub(crate) fn new(
        absolute: AbsoluteOffset,
        relative: RelativeOffset,
        range: CellRange,
    ) -> Self {
        Self {
            absolute,
            relative,
            range,
        }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`].
    pub fn absolute_offset(&self) -> AbsoluteOffset {
        self.absolute
    }

    /// Returns the [`RelativeOffset`] of the current [`Viewport`].
    pub fn relative_offset(&self) -> RelativeOffset {
        self.relative
    }

    /// Returns the [`CellRange`] of the current [`Viewport`].
    pub fn cell_range(&self) -> CellRange {
        self.range
    }
}

impl std::fmt::Display for Viewport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Viewport{{abs:({},{}), rel:({},{}), range:{}",
            self.absolute.x, self.absolute.y, self.relative.x, self.relative.y, self.range
        )
    }
}

/// Properties of a scrollbar within a [`GridScrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Properties {
    width: f32,
    margin: f32,
    scroller_width: f32,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            width: 10.0,
            margin: 0.0,
            scroller_width: 10.0,
        }
    }
}

impl Properties {
    /// Creates new [`Properties`] for use in a [`GridScrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the scrollbar width of the [`GridScrollable`] .
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(0.0);
        self
    }

    /// Sets the scrollbar margin of the [`GridScrollable`] .
    pub fn margin(mut self, margin: impl Into<Pixels>) -> Self {
        self.margin = margin.into().0;
        self
    }

    /// Sets the scroller width of the [`GridScrollable`] .
    pub fn scroller_width(mut self, scroller_width: impl Into<Pixels>) -> Self {
        self.scroller_width = scroller_width.into().0.max(0.0);
        self
    }

    /// Measurement across the scrollbar (total width of vertical or total height of horizontal)
    fn across(&self) -> f32 {
        self.width.max(self.scroller_width) + 2.0 * self.margin
    }
}
