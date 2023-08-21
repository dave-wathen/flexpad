//! Navigate an endless amount of content with a scrollbar.
use std::rc::Rc;

use iced::advanced::widget::{self, tree, Operation, Tree};
use iced::advanced::{layout, renderer, Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::mouse;
use iced::overlay;
use iced::touch;
use iced::widget::scrollable::{Scrollbar, StyleSheet};
use iced::Command;
use iced::{keyboard, window};
use iced::{Background, Color, Element, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::sequence::Rounding;
use crate::{operation, CellRange, Grid, SumSeq};

// TODO: Visible only contents to allow large grids
// TODO: Programatic scrolling to row/column?

/// A widget that can display a large [`Grid`] with scrollbars
#[allow(missing_debug_implementations)]
pub struct GridScrollable<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
    <Renderer::Theme as crate::style::StyleSheet>::Style: Clone,
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
    <Renderer::Theme as crate::style::StyleSheet>::Style: Clone,
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

    fn constituents(&self, tree: &Tree, layout: Layout<'_>) -> Constituents {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        let mut grid_children_layouts = content_layout.children();
        let row_heads_bounds = self
            .content
            .row_heads
            .is_some()
            .then(|| grid_children_layouts.next().unwrap().bounds());
        let column_heads_bounds = self
            .content
            .column_heads
            .is_some()
            .then(|| grid_children_layouts.next().unwrap().bounds());

        // Grid cell bounds is the portion of the Grid that contains cells
        let heads_adjustment = match (row_heads_bounds, column_heads_bounds) {
            (None, None) => Vector::ZERO,
            (None, Some(chb)) => Vector::new(0.0, chb.height),
            (Some(rhb), None) => Vector::new(rhb.width, 0.0),
            (Some(rhb), Some(chb)) => Vector::new(rhb.width, chb.height),
        };
        let grid_cells_bounds = Rectangle::new(
            content_bounds.position() + heads_adjustment,
            (content_bounds.size() - heads_adjustment.into()).max(Size::ZERO),
        );

        // Initial sizes for scrollbars
        let (x_height, y_width) = (self.horizontal.across(), self.vertical.across());
        let (x_width, y_height) = (
            (bounds.width - row_heads_bounds.map_or(0.0, |b| b.width)).max(0.0),
            (bounds.height - column_heads_bounds.map_or(0.0, |b| b.height)).max(0.0),
        );

        // Determine which are active
        let (x_active, y_active) = if content_bounds.width > bounds.width {
            (true, content_bounds.height > y_height - x_height)
        } else if content_bounds.height > bounds.height {
            (content_bounds.width > x_width - y_width, true)
        } else {
            (false, false)
        };

        // Adjust sizes if necessary
        let (x_width, x_height, y_width, y_height) = match (x_active, y_active) {
            (true, true) => (
                (x_width - y_width).max(0.0),
                x_height,
                y_width,
                (y_height - x_height).max(0.0),
            ),
            (true, false) => (x_width, x_height, 0.0, 0.0),
            (false, true) => (0.0, 0.0, y_width, y_height),
            (false, false) => (0.0, 0.0, 0.0, 0.0),
        };

        // Calculate the sub-viewports for the grid (row/column heads and cells)
        let scroll_bars_adjustment = Size::new(y_width, x_height);
        let cells_viewport = Rectangle::new(
            bounds.position() + heads_adjustment,
            (bounds.size() - heads_adjustment.into() - scroll_bars_adjustment).max(Size::ZERO),
        );

        let row_heads_viewport = row_heads_bounds.map(|b| {
            Rectangle::new(
                Point::new(bounds.x, cells_viewport.y),
                Size::new(b.width, cells_viewport.height),
            )
        });
        let column_heads_viewport = column_heads_bounds.map(|b| {
            Rectangle::new(
                Point::new(cells_viewport.x, bounds.y),
                Size::new(cells_viewport.width, b.height),
            )
        });
        let corner_viewport = match (row_heads_viewport, column_heads_viewport) {
            (Some(rhv), Some(chv)) => Some(Rectangle::new(
                bounds.position(),
                Size::new(rhv.width, chv.height),
            )),
            _ => None,
        };

        let info = (*self.content.info).borrow();
        let grid_measures = GridMeasures {
            cells_bounds: grid_cells_bounds,
            row_heights: Rc::clone(&info.row_heights),
            column_widths: Rc::clone(&info.column_widths),
            vertical_granularity: self.vertical_granularity,
            horizontal_granularity: self.horizontal_granularity,
        };

        // Calculate the offset
        let scales = ScrollScales {
            viewport: cells_viewport,
            grid_measures: grid_measures.clone(),
        };
        let offset = state.offset(scales);

        let y_scrollbar = y_active.then(|| {
            let Properties {
                width,
                margin: _margin,
                scroller_width,
            } = self.vertical;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: cells_viewport.x + cells_viewport.width,
                y: cells_viewport.y,
                width: y_width,
                height: y_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: cells_viewport.x + cells_viewport.width + (y_width - width) / 2.0,
                y: cells_viewport.y,
                width,
                height: y_height,
            };

            let ratio = y_height / grid_cells_bounds.height;
            // min height for easier grabbing with super tall content
            let scroller_height = (y_height * ratio).max(2.0);
            let scroller_offset = offset.y * ratio;

            let scroller_bounds = Rectangle {
                x: cells_viewport.x + cells_viewport.width + (y_width - scroller_width) / 2.0,
                y: (scrollbar_bounds.y + scroller_offset).max(0.0),
                width: scroller_width,
                height: scroller_height,
            };

            GridScrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: GridScroller {
                    bounds: scroller_bounds,
                },
            }
        });

        let x_scrollbar = x_active.then(|| {
            let Properties {
                width,
                margin: _margin,
                scroller_width,
            } = self.horizontal;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: cells_viewport.x,
                y: cells_viewport.y + cells_viewport.height,
                width: x_width,
                height: x_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: cells_viewport.x,
                y: cells_viewport.y + cells_viewport.height + (x_height - width) / 2.0,
                width: x_width,
                height: width,
            };

            let ratio = x_width / grid_cells_bounds.width;
            // min width for easier grabbing with extra wide content
            let scroller_length = (x_width * ratio).max(2.0);
            let scroller_offset = offset.x * ratio;

            let scroller_bounds = Rectangle {
                x: (scrollbar_bounds.x + scroller_offset).max(0.0),
                y: cells_viewport.y + cells_viewport.height + (x_height - scroller_width) / 2.0,
                width: scroller_length,
                height: scroller_width,
            };

            GridScrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: GridScroller {
                    bounds: scroller_bounds,
                },
            }
        });

        let bars_fill_in = (x_active && y_active).then(|| {
            Rectangle::new(
                Point::new(
                    cells_viewport.x + cells_viewport.width,
                    cells_viewport.y + cells_viewport.height,
                ),
                Size::new(y_width, x_height),
            )
        });

        let row_heads_fill_in = match (x_scrollbar, row_heads_bounds) {
            (Some(xsb), Some(rhb)) => Some(Rectangle::new(
                Point::new(bounds.x, xsb.bounds.y),
                Size::new(rhb.width, xsb.bounds.height),
            )),
            _ => None,
        };

        let column_heads_fill_in = match (y_scrollbar, column_heads_bounds) {
            (Some(ysb), Some(chb)) => Some(Rectangle::new(
                Point::new(ysb.bounds.x, bounds.y),
                Size::new(ysb.bounds.width, chb.height),
            )),
            _ => None,
        };

        Constituents {
            y: y_scrollbar,
            x: x_scrollbar,
            bars_fill_in,
            row_heads_fill_in,
            column_heads_fill_in,
            cells_viewport,
            row_heads_viewport,
            column_heads_viewport,
            corner_viewport,
            grid_measures,
        }
    }
}

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer> for GridScrollable<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
    <Renderer::Theme as crate::style::StyleSheet>::Style: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
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
        //let state = tree.state.downcast_mut::<State>();

        //operation.scrollable(state, self.id.as_ref().map(|id| &id.0));

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
        let constituents = self.constituents(tree, layout);
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content_layout = layout.children().next().unwrap();

        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = constituents.is_mouse_over(cursor);

        let event_status = {
            let cursor = match cursor_over_scrollable {
                Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                    mouse::Cursor::Available(cursor_position + state.offset(constituents.scales()))
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
            Event::Window(window::Event::Resized {
                width: _,
                height: _,
            }) => {
                notify_on_viewport_change(
                    state,
                    &self.on_viewport_change,
                    constituents.scales(),
                    shell,
                );
            }
            Event::Window(window::Event::RedrawRequested(_)) if state.last_notified.is_none() => {
                notify_on_viewport_change(
                    state,
                    &self.on_viewport_change,
                    constituents.scales(),
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

                state.scroll(delta, constituents.scales());

                notify_on_viewport_change(
                    state,
                    &self.on_viewport_change,
                    constituents.scales(),
                    shell,
                );

                return event::Status::Captured;
            }
            Event::Touch(event)
                if state.scroll_area_touched_at.is_some()
                    || !mouse_over_y_scrollbar && !mouse_over_x_scrollbar =>
            {
                match event {
                    touch::Event::FingerPressed { .. } => {
                        let Some(cursor_position) = cursor.position() else {
                            return event::Status::Ignored
                        };

                        state.scroll_area_touched_at = Some(cursor_position);
                    }
                    touch::Event::FingerMoved { .. } => {
                        if let Some(scroll_box_touched_at) = state.scroll_area_touched_at {
                            let Some(cursor_position) = cursor.position() else {
                                return event::Status::Ignored
                            };

                            let delta = Vector::new(
                                cursor_position.x - scroll_box_touched_at.x,
                                cursor_position.y - scroll_box_touched_at.y,
                            );

                            state.scroll(delta, constituents.scales());

                            state.scroll_area_touched_at = Some(cursor_position);

                            notify_on_viewport_change(
                                state,
                                &self.on_viewport_change,
                                constituents.scales(),
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
                    if let Some(scrollbar) = constituents.y {
                        let Some(cursor_position) = cursor.position() else {
                            return event::Status::Ignored
                        };

                        state.scroll_y_to(
                            scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                            constituents.scales(),
                        );

                        notify_on_viewport_change(
                            state,
                            &self.on_viewport_change,
                            constituents.scales(),
                            shell,
                        );

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        } else if mouse_over_y_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored
                    };

                    if let (Some(scroller_grabbed_at), Some(scrollbar)) = (
                        constituents.grab_y_scroller(cursor_position),
                        constituents.y,
                    ) {
                        state.scroll_y_to(
                            scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                            constituents.scales(),
                        );

                        state.y_scroller_grabbed_at = Some(scroller_grabbed_at);

                        notify_on_viewport_change(
                            state,
                            &self.on_viewport_change,
                            constituents.scales(),
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
                        return event::Status::Ignored
                    };

                    if let Some(scrollbar) = constituents.x {
                        state.scroll_x_to(
                            scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                            constituents.scales(),
                        );

                        notify_on_viewport_change(
                            state,
                            &self.on_viewport_change,
                            constituents.scales(),
                            shell,
                        );
                    }

                    return event::Status::Captured;
                }
                _ => {}
            }
        } else if mouse_over_x_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored
                    };

                    if let (Some(scroller_grabbed_at), Some(scrollbar)) = (
                        constituents.grab_x_scroller(cursor_position),
                        constituents.x,
                    ) {
                        state.scroll_x_to(
                            scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                            constituents.scales(),
                        );

                        state.x_scroller_grabbed_at = Some(scroller_grabbed_at);

                        notify_on_viewport_change(
                            state,
                            &self.on_viewport_change,
                            constituents.scales(),
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
        let constituents = self.constituents(tree, layout);
        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let cursor_over_scrollable = cursor.position_over(bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = constituents.is_mouse_over(cursor);

        let offset = state.offset(constituents.scales());

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + offset)
            }
            _ => mouse::Cursor::Unavailable,
        };

        if constituents.can_scroll() {
            // Draw grid content
            renderer.with_layer(constituents.cells_viewport, |renderer| {
                renderer.with_translation(Vector::ZERO - offset, |renderer| {
                    self.content.draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        content_layout,
                        cursor,
                        &Rectangle::new(
                            constituents.cells_viewport.position() + offset,
                            constituents.cells_viewport.size(),
                        ),
                    );
                });
            });

            // Draw row heads
            if let Some(rh_viewport) = constituents.row_heads_viewport {
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
            if let Some(ch_viewport) = constituents.column_heads_viewport {
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
            if let Some(c_viewport) = constituents.corner_viewport {
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

            let draw_scrollbar = |renderer: &mut Renderer,
                                  style: Scrollbar,
                                  scrollbar: &GridScrollbar| {
                //track
                if scrollbar.bounds.width > 0.0
                    && scrollbar.bounds.height > 0.0
                    && (style.background.is_some()
                        || (style.border_color != Color::TRANSPARENT && style.border_width > 0.0))
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.bounds,
                            border_radius: style.border_radius,
                            border_width: style.border_width,
                            border_color: style.border_color,
                        },
                        style
                            .background
                            .unwrap_or(Background::Color(Color::TRANSPARENT)),
                    );
                }

                //thumb
                if scrollbar.scroller.bounds.width > 0.0
                    && scrollbar.scroller.bounds.height > 0.0
                    && (style.scroller.color != Color::TRANSPARENT
                        || (style.scroller.border_color != Color::TRANSPARENT
                            && style.scroller.border_width > 0.0))
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.scroller.bounds,
                            border_radius: style.scroller.border_radius,
                            border_width: style.scroller.border_width,
                            border_color: style.scroller.border_color,
                        },
                        style.scroller.color,
                    );
                }
            };

            renderer.with_layer(
                Rectangle {
                    width: bounds.width + 2.0,
                    height: bounds.height + 2.0,
                    ..bounds
                },
                |renderer| {
                    //draw y scrollbar
                    if let Some(scrollbar) = constituents.y {
                        let style = if state.y_scroller_grabbed_at.is_some() {
                            theme.dragging(&self.style)
                        } else if cursor_over_scrollable.is_some() {
                            theme.hovered(&self.style, mouse_over_y_scrollbar)
                        } else {
                            theme.active(&self.style)
                        };

                        draw_scrollbar(renderer, style, &scrollbar);
                    }

                    //draw x scrollbar
                    if let Some(scrollbar) = constituents.x {
                        let style = if state.x_scroller_grabbed_at.is_some() {
                            theme.dragging_horizontal(&self.style)
                        } else if cursor_over_scrollable.is_some() {
                            theme.hovered_horizontal(&self.style, mouse_over_x_scrollbar)
                        } else {
                            theme.active_horizontal(&self.style)
                        };

                        draw_scrollbar(renderer, style, &scrollbar);
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
                    for fill_in in constituents.fill_ins() {
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
                    x: bounds.x + offset.x,
                    y: bounds.y + offset.y,
                    ..bounds
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
        let constituents = self.constituents(tree, layout);
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content_layout = layout.children().next().unwrap();

        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = constituents.is_mouse_over(cursor);

        if (mouse_over_x_scrollbar || mouse_over_y_scrollbar) || state.scrollers_grabbed() {
            mouse::Interaction::Idle
        } else {
            let offset = state.offset(constituents.scales());

            let cursor = match cursor_over_scrollable {
                Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                    mouse::Cursor::Available(cursor_position + offset)
                }
                _ => mouse::Cursor::Unavailable,
            };

            self.content.mouse_interaction(
                &tree.children[0],
                content_layout,
                cursor,
                &Rectangle {
                    y: bounds.y + offset.y,
                    x: bounds.x + offset.x,
                    ..bounds
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
        let constituents = self.constituents(tree, layout);
        self.content
            .overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
            .map(|overlay| {
                let offset = tree
                    .state
                    .downcast_ref::<State>()
                    .offset(constituents.scales());

                overlay.translate(Vector::new(-offset.x, -offset.y))
            })
    }
}

impl<'a, Message, Renderer> From<GridScrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    Renderer::Theme: crate::style::StyleSheet,
    <Renderer::Theme as crate::style::StyleSheet>::Style: Clone,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Granularity {
    #[default]
    Discrete,
    Continuous,
}

/// Produces a [`Command`] that snaps the [`GridScrollable`] with the given [`Id`]
/// to the provided `percentage` along the x & y axis.
#[allow(dead_code)]
pub fn snap_to<Message: 'static>(id: Id, offset: operation::RelativeOffset) -> Command<Message> {
    Command::widget(operation::snap_to(id.0, offset))
}

/// Produces a [`Command`] that scrolls the [`GridScrollable`] with the given [`Id`]
/// to the provided [`AbsoluteOffset`] along the x & y axis.
#[allow(dead_code)]
pub fn scroll_to<Message: 'static>(id: Id, offset: operation::AbsoluteOffset) -> Command<Message> {
    Command::widget(operation::scroll_to(id.0, offset))
}

fn notify_on_viewport_change<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
    scales: ScrollScales,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_scroll) = on_scroll {
        if !scales.can_scroll_x() && !scales.can_scroll_y() {
            return;
        }

        let viewport = Viewport::new(state.offset_x, state.offset_y, scales);

        // Don't publish redundant viewports to shell
        if let Some(last_notified) = state.last_notified {
            let last_relative_offset = last_notified.relative_offset();
            let current_relative_offset = viewport.relative_offset();

            let last_absolute_offset = last_notified.absolute_offset();
            let current_absolute_offset = viewport.absolute_offset();

            let unchanged =
                |a: f32, b: f32| (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan());

            if unchanged(last_relative_offset.x, current_relative_offset.x)
                && unchanged(last_relative_offset.y, current_relative_offset.y)
                && unchanged(last_absolute_offset.x, current_absolute_offset.x)
                && unchanged(last_absolute_offset.y, current_absolute_offset.y)
            {
                return;
            }
        }

        shell.publish(on_scroll(viewport));
        state.last_notified = Some(viewport);
    }
}

/// The local state of a [`GridScrollable`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    scroll_area_touched_at: Option<Point>,
    offset_y: Offset,
    y_scroller_grabbed_at: Option<f32>,
    offset_x: Offset,
    x_scroller_grabbed_at: Option<f32>,
    keyboard_modifiers: keyboard::Modifiers,
    last_notified: Option<Viewport>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scroll_area_touched_at: None,
            offset_y: Offset::Absolute(0.0),
            y_scroller_grabbed_at: None,
            offset_x: Offset::Absolute(0.0),
            x_scroller_grabbed_at: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_notified: None,
        }
    }
}

impl operation::GridScrollable for State {
    fn snap_to(&mut self, offset: operation::RelativeOffset) {
        State::snap_to(self, offset);
    }

    fn scroll_to(&mut self, offset: operation::AbsoluteOffset) {
        State::scroll_to(self, offset)
    }
}

#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, dimension: &ScrollScale) -> f32 {
        match self {
            Offset::Absolute(absolute) => dimension.quantize(absolute),
            Offset::Relative(percentage) => dimension.percentage(percentage),
        }
    }
}

/// The current [`Viewport`] of the [`GridScrollable`].
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    absolute: operation::AbsoluteOffset,
    relative: operation::RelativeOffset,
    range: CellRange,
}

impl Viewport {
    fn new(offset_x: Offset, offset_y: Offset, scales: ScrollScales) -> Self {
        let x_scale = scales.x();
        let y_scale = scales.y();

        let x = offset_x.absolute(&x_scale);
        let y = offset_y.absolute(&y_scale);
        let absolute = operation::AbsoluteOffset { x, y };

        let start_column = x_scale.discretes.index_of_sum(x, Rounding::Up).unwrap_or(0) as u32;
        let start_row = y_scale.discretes.index_of_sum(y, Rounding::Up).unwrap_or(0) as u32;
        let end_column = x_scale
            .discretes
            .index_of_sum(x + scales.viewport.width, Rounding::Down)
            .unwrap_or(x_scale.discretes.len() - 1) as u32;
        let end_row = y_scale
            .discretes
            .index_of_sum(y + scales.viewport.height, Rounding::Down)
            .unwrap_or(y_scale.discretes.len() - 1) as u32;
        let range = CellRange::new((start_row, start_column), (end_row, end_column));

        let x = x_scale.absolute_to_relative(x);
        let y = y_scale.absolute_to_relative(y);
        let relative = operation::RelativeOffset { x, y };

        Self {
            absolute,
            relative,
            range,
        }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`].
    pub fn absolute_offset(&self) -> operation::AbsoluteOffset {
        self.absolute
    }

    /// Returns the [`RelativeOffset`] of the current [`Viewport`].
    pub fn relative_offset(&self) -> operation::RelativeOffset {
        self.relative
    }

    /// Returns the [`CellRange`] of the current [`Viewport`].
    pub fn cell_range(&self) -> CellRange {
        self.range
    }
}

impl State {
    /// Creates a new [`State`] with the scrollbar(s) at the beginning.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the context.
    fn scroll(&mut self, delta: Vector<f32>, scales: ScrollScales) {
        if scales.can_scroll_y() {
            let dimension = scales.y();
            self.offset_y =
                Offset::Absolute(dimension.quantize(self.offset_y.absolute(&dimension) - delta.y));
        };

        if scales.can_scroll_x() {
            let dimension = scales.x();
            self.offset_x =
                Offset::Absolute(dimension.quantize(self.offset_x.absolute(&dimension) - delta.x));
        }
    }

    /// Scrolls the [`GridScrollable`] to a relative amount along the y axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    fn scroll_y_to(&mut self, percentage: f32, scales: ScrollScales) {
        self.offset_y = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(scales);
    }

    /// Scrolls the [`GridScrollable`] to a relative amount along the x axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    fn scroll_x_to(&mut self, percentage: f32, scales: ScrollScales) {
        self.offset_x = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(scales);
    }

    /// Snaps the scroll position to a [`RelativeOffset`].
    fn snap_to(&mut self, offset: operation::RelativeOffset) {
        self.offset_x = Offset::Relative(offset.x.clamp(0.0, 1.0));
        self.offset_y = Offset::Relative(offset.y.clamp(0.0, 1.0));
    }

    /// Scroll to the provided [`AbsoluteOffset`].
    fn scroll_to(&mut self, offset: operation::AbsoluteOffset) {
        self.offset_x = Offset::Absolute(offset.x.max(0.0));
        self.offset_y = Offset::Absolute(offset.y.max(0.0));
    }

    /// Unsnaps the current scroll position, if snapped, given the context.
    fn unsnap(&mut self, scales: ScrollScales) {
        self.offset_x = Offset::Absolute(self.offset_x.absolute(&scales.x()));
        self.offset_y = Offset::Absolute(self.offset_y.absolute(&scales.y()));
    }

    /// Returns the scrolling offset of the [`State`], given the context.
    fn offset(&self, scales: ScrollScales) -> Vector {
        Vector::new(
            self.offset_x.absolute(&scales.x()),
            self.offset_y.absolute(&scales.y()),
        )
    }

    /// Returns whether any scroller is currently grabbed or not.
    pub fn scrollers_grabbed(&self) -> bool {
        self.x_scroller_grabbed_at.is_some() || self.y_scroller_grabbed_at.is_some()
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

#[derive(Debug, Clone)]
struct ScrollScale {
    viewport: f32,
    onto: f32,
    discretes: Rc<SumSeq>,
    granularity: Granularity,
}

impl ScrollScale {
    fn quantize(&self, value: f32) -> f32 {
        let value = value.clamp(0.0, (self.onto - self.viewport).max(0.0));
        if self.granularity == Granularity::Discrete {
            let index = self
                .discretes
                .index_of_sum(value, Rounding::Down)
                .unwrap_or(0);
            let start = self.discretes.sum_to(index);
            let end = self.discretes.sum_to(index + 1);
            if value > (start + end) / 2.0 {
                end
            } else {
                start
            }
        } else {
            value
        }
    }

    fn percentage(&self, percentage: f32) -> f32 {
        self.quantize((self.onto - self.viewport) * percentage)
    }

    fn absolute_to_relative(&self, a: f32) -> f32 {
        a / (self.onto - self.viewport)
    }
}

#[derive(Debug, Clone)]
struct ScrollScales {
    viewport: Rectangle,
    grid_measures: GridMeasures,
}

impl ScrollScales {
    fn can_scroll_x(&self) -> bool {
        self.viewport.width < self.grid_measures.cells_bounds.width
    }

    fn can_scroll_y(&self) -> bool {
        self.viewport.height < self.grid_measures.cells_bounds.height
    }

    fn x(&self) -> ScrollScale {
        ScrollScale {
            viewport: self.viewport.width,
            onto: self.grid_measures.cells_bounds.width,
            discretes: Rc::clone(&self.grid_measures.column_widths),
            granularity: self.grid_measures.horizontal_granularity,
        }
    }

    fn y(&self) -> ScrollScale {
        ScrollScale {
            viewport: self.viewport.height,
            onto: self.grid_measures.cells_bounds.height,
            discretes: Rc::clone(&self.grid_measures.row_heights),
            granularity: self.grid_measures.vertical_granularity,
        }
    }
}

#[derive(Debug, Clone)]
struct GridMeasures {
    cells_bounds: Rectangle,
    row_heights: Rc<SumSeq>,
    column_widths: Rc<SumSeq>,
    vertical_granularity: Granularity,
    horizontal_granularity: Granularity,
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
struct Constituents {
    y: Option<GridScrollbar>,
    x: Option<GridScrollbar>,
    bars_fill_in: Option<Rectangle>,
    row_heads_fill_in: Option<Rectangle>,
    column_heads_fill_in: Option<Rectangle>,
    cells_viewport: Rectangle,
    row_heads_viewport: Option<Rectangle>,
    column_heads_viewport: Option<Rectangle>,
    corner_viewport: Option<Rectangle>,
    grid_measures: GridMeasures,
}

impl Constituents {
    fn scales(&self) -> ScrollScales {
        ScrollScales {
            viewport: self.cells_viewport,
            grid_measures: self.grid_measures.clone(),
        }
    }

    fn is_mouse_over(&self, cursor: mouse::Cursor) -> (bool, bool) {
        if let Some(cursor_position) = cursor.position() {
            (
                self.y
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
                self.x
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
            )
        } else {
            (false, false)
        }
    }

    fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.y.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.y - scrollbar.scroller.bounds.y)
                        / scrollbar.scroller.bounds.height
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.x.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.x - scrollbar.scroller.bounds.x)
                        / scrollbar.scroller.bounds.width
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn can_scroll(&self) -> bool {
        self.y.is_some() || self.x.is_some()
    }

    fn fill_ins(&self) -> impl Iterator<Item = Rectangle> + '_ {
        self.bars_fill_in
            .iter()
            .chain(self.row_heads_fill_in.iter())
            .chain(self.column_heads_fill_in.iter())
            .copied()
    }
}

/// The scrollbar of a [`GridScrollable`].
#[derive(Debug, Copy, Clone)]
struct GridScrollbar {
    /// The total bounds of the [`Scrollbar`], including the scrollbar, the scroller,
    /// and the scrollbar margin.
    total_bounds: Rectangle,

    /// The bounds of just the [`Scrollbar`].
    bounds: Rectangle,

    /// The state of this scrollbar's [`Scroller`].
    scroller: GridScroller,
}

impl GridScrollbar {
    /// Returns whether the mouse is over the scrollbar or not.
    fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.total_bounds.contains(cursor_position)
    }

    /// Returns the y-axis scrolled percentage from the cursor position.
    fn scroll_percentage_y(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
        if cursor_position.x < 0.0 && cursor_position.y < 0.0 {
            // cursor position is unavailable! Set to either end or beginning of scrollbar depending
            // on where the thumb currently is in the track
            (self.scroller.bounds.y / self.total_bounds.height).round()
        } else {
            (cursor_position.y - self.bounds.y - self.scroller.bounds.height * grabbed_at)
                / (self.bounds.height - self.scroller.bounds.height)
        }
    }

    /// Returns the x-axis scrolled percentage from the cursor position.
    fn scroll_percentage_x(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
        if cursor_position.x < 0.0 && cursor_position.y < 0.0 {
            (self.scroller.bounds.x / self.total_bounds.width).round()
        } else {
            (cursor_position.x - self.bounds.x - self.scroller.bounds.width * grabbed_at)
                / (self.bounds.width - self.scroller.bounds.width)
        }
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
struct GridScroller {
    /// The bounds of the [`Scroller`].
    bounds: Rectangle,
}
