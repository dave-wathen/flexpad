use std::{rc::Rc, time::Instant};

use iced::{
    advanced::{
        renderer,
        widget::{tree, Tree},
        Layout, Shell,
    },
    keyboard, mouse,
    widget::scrollable::{AbsoluteOffset, RelativeOffset, Scrollbar},
    Background, Color, Point, Rectangle, Size, Vector,
};

use crate::{grid::state::GridState, sequence::Rounding, CellRange, StyleSheet, SumSeq, Viewport};

use super::Properties;

/// The local state of a [`GridScrollable`].
#[derive(Debug, Clone)]
pub struct GridScrollableState {
    horizontal_granularity: Granularity,
    vertical_granularity: Granularity,

    pub scroll_area_touched_at: Option<Point>,
    pub y_scroller_grabbed_at: Option<f32>,
    pub x_scroller_grabbed_at: Option<f32>,
    pub keyboard_modifiers: keyboard::Modifiers,
    unused_x_delta: f32,
    unused_y_delta: f32,
    unused_delta_last_updated: Instant,

    cells_bounds: Rectangle,
    row_heights: Rc<SumSeq>,
    column_widths: Rc<SumSeq>,
    last_notified: Option<Viewport>,
}

impl GridScrollableState {
    pub fn new(horizontal_granularity: Granularity, vertical_granularity: Granularity) -> Self {
        Self {
            horizontal_granularity,
            vertical_granularity,
            ..Default::default()
        }
    }

    pub fn calculate_parts_and_update(
        &mut self,
        bounds: Rectangle,
        x_properties: Properties,
        y_properties: Properties,
        grid_tree: &Tree,
        grid_layout: Layout<'_>,
    ) -> ScrollableParts {
        let result =
            self.calculate_parts(bounds, x_properties, y_properties, grid_tree, grid_layout);
        if self.cells_bounds.size() != result.cells_viewport.size() {
            self.cells_bounds.width = result.cells_viewport.size().width;
            self.cells_bounds.height = result.cells_viewport.size().height;
        }

        result
    }

    pub fn calculate_parts(
        &self,
        bounds: Rectangle,
        x_properties: Properties,
        y_properties: Properties,
        grid_tree: &Tree,
        grid_layout: Layout<'_>,
    ) -> ScrollableParts {
        let content_bounds = grid_layout.bounds();
        let grid_parts = GridPartsBounds::new(grid_tree, grid_layout);

        // Initial sizes for scrollbars
        let (x_height, y_width) = (x_properties.across(), y_properties.across());
        let (x_width, y_height) = (
            (bounds.width - grid_parts.row_heads_width()).max(0.0),
            (bounds.height - grid_parts.column_heads_height()).max(0.0),
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
        let cells_viewport = Rectangle::new(
            grid_parts.cells.position(),
            Size::new(
                bounds.size().width - grid_parts.row_heads_width() - y_width,
                bounds.size().height - grid_parts.column_heads_height() - x_height,
            ),
        );

        let row_heads_viewport = grid_parts.row_heads.map(|b| {
            Rectangle::new(
                Point::new(bounds.x, cells_viewport.y),
                Size::new(b.width, cells_viewport.height),
            )
        });
        let column_heads_viewport = grid_parts.column_heads.map(|b| {
            Rectangle::new(
                Point::new(cells_viewport.x, bounds.y),
                Size::new(cells_viewport.width, b.height),
            )
        });
        let corner_viewport = grid_parts.corner;

        let offset = self.absolute_offset();

        let y_scrollbar = y_active.then(|| {
            let Properties {
                width,
                margin: _margin,
                scroller_width,
            } = y_properties;

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

            let ratio = y_height / grid_parts.cells.height;
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
            } = x_properties;

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

            let ratio = x_width / grid_parts.cells.width;
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

        ScrollableParts {
            y_scrollbar,
            x_scrollbar,
            cells_viewport,
            row_heads_viewport,
            column_heads_viewport,
            corner_viewport,
        }
    }

    pub fn is_x_scroller_grabbed(&self) -> bool {
        self.x_scroller_grabbed_at.is_some()
    }

    pub fn is_y_scroller_grabbed(&self) -> bool {
        self.y_scroller_grabbed_at.is_some()
    }

    pub fn is_a_scroller_grabbed(&self) -> bool {
        self.is_x_scroller_grabbed() || self.is_y_scroller_grabbed()
    }

    fn set_x_offset(&mut self, x: f32, viewport_width: f32, column_widths: &Rc<SumSeq>) -> bool {
        self.column_widths = Rc::clone(column_widths);
        let quantized = quantize(
            x,
            viewport_width,
            column_widths.clone(),
            self.horizontal_granularity,
        );
        if quantized != self.cells_bounds.x {
            self.cells_bounds.x = quantized;
            self.cells_bounds.width = viewport_width;
            true
        } else {
            false
        }
    }

    fn set_y_offset(&mut self, y: f32, viewport_height: f32, row_heights: &Rc<SumSeq>) -> bool {
        self.row_heights = Rc::clone(row_heights);
        let quantized = quantize(
            y,
            viewport_height,
            row_heights.clone(),
            self.vertical_granularity,
        );
        if quantized != self.cells_bounds.y {
            self.cells_bounds.y = quantized;
            self.cells_bounds.height = viewport_height;
            true
        } else {
            false
        }
    }

    pub fn visible_range(&self) -> CellRange {
        let x = self.cells_bounds.x;
        let y = self.cells_bounds.y;
        let width = self.cells_bounds.width;
        let height = self.cells_bounds.height;

        let start_column = self
            .column_widths
            .index_of_sum(x, Rounding::Up)
            .unwrap_or(0) as u32;
        let start_row = self.row_heights.index_of_sum(y, Rounding::Up).unwrap_or(0) as u32;
        let end_column = self
            .column_widths
            .index_of_sum(x + width, Rounding::Down)
            .unwrap_or(self.column_widths.len() - 1) as u32;
        let end_row = self
            .row_heights
            .index_of_sum(y + height, Rounding::Down)
            .unwrap_or(self.row_heights.len() - 1) as u32;

        CellRange::new((start_row, start_column), (end_row, end_column))
    }

    pub fn viewport(&self) -> Viewport {
        let width = self.cells_bounds.width;
        let height = self.cells_bounds.height;

        let cells_width = self.column_widths.sum();
        let cells_height = self.row_heights.sum();
        let can_scroll_x = cells_width > width;
        let can_scroll_y = cells_height > height;

        let Vector { x, y } = self.absolute_offset();
        let absolute = AbsoluteOffset { x, y };

        let scrollable_width = (cells_width - width).max(0.0);
        let scrollable_height = (cells_height - height).max(0.0);
        let rel_x = if can_scroll_x {
            absolute.x / scrollable_width
        } else {
            0.0
        };
        let rel_y = if can_scroll_y {
            absolute.y / scrollable_height
        } else {
            0.0
        };
        let relative = RelativeOffset { x: rel_x, y: rel_y };
        let range = self.visible_range();

        Viewport::new(absolute, relative, range)
    }

    pub fn scroll(&mut self, delta: Vector<f32>, viewport: Rectangle, grid_state: &GridState) {
        if self.unused_delta_last_updated.elapsed().as_millis() > 1000 {
            self.unused_x_delta = 0.0;
            self.unused_y_delta = 0.0;
        }

        let can_scroll_x = grid_state.column_widths.sum() > viewport.width;
        let can_scroll_y = grid_state.row_heights.sum() > viewport.height;

        let mut x_changed = false;
        let mut y_changed = false;

        if can_scroll_x {
            let new_x = self.cells_bounds.x - (delta.x + self.unused_x_delta);
            x_changed = self.set_x_offset(new_x, viewport.width, &grid_state.column_widths);
        };
        if can_scroll_y {
            let new_y = self.cells_bounds.y - (delta.y + self.unused_y_delta);
            y_changed = self.set_y_offset(new_y, viewport.height, &grid_state.row_heights);
        };

        if x_changed {
            self.unused_x_delta = 0.0;
        } else {
            self.unused_x_delta += delta.x;
            self.unused_delta_last_updated = Instant::now();
        }
        if y_changed {
            self.unused_y_delta = 0.0;
        } else {
            self.unused_y_delta += delta.y;
            self.unused_delta_last_updated = Instant::now();
        }
    }

    /// Scrolls the [`GridScrollable`] to a relative amount along the y axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_y_to(&mut self, percentage: f32, viewport_height: f32, row_heights: &Rc<SumSeq>) {
        let scrollable_height = (row_heights.sum() - viewport_height).max(0.0);
        let offset = scrollable_height * percentage;
        self.set_y_offset(offset, viewport_height, row_heights);
    }

    /// Scrolls the [`GridScrollable`] to a relative amount along the x axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_x_to(
        &mut self,
        percentage: f32,
        viewport_width: f32,
        column_widths: &Rc<SumSeq>,
    ) {
        let scrollable_width = (column_widths.sum() - viewport_width).max(0.0);
        let offset = scrollable_width * percentage;
        self.set_x_offset(offset, viewport_width, column_widths);
    }

    pub fn ensure_column_visible(&mut self, column: u32) {
        let required = self.column_widths.sum_to(column as usize)
            ..=self.column_widths.sum_to(column as usize + 1);

        if self.cells_bounds.x > *required.start() {
            let new_x = self.column_widths.sum_to(column as usize);
            let column_widths = self.column_widths.clone();
            self.set_x_offset(new_x, self.cells_bounds.width, &column_widths);
        } else if self.cells_bounds.x + self.cells_bounds.width < *required.end() {
            let first_column = self
                .column_widths
                .index_of_sum(required.end() - self.cells_bounds.width, Rounding::Up)
                .map(|i| i + 1)
                .unwrap_or(column as usize);
            let new_x = self.column_widths.sum_to(first_column);
            let column_widths = self.column_widths.clone();
            self.set_x_offset(new_x, self.cells_bounds.width, &column_widths);
        }
    }

    pub fn ensure_row_visible(&mut self, row: u32) {
        let required =
            self.row_heights.sum_to(row as usize)..=self.row_heights.sum_to(row as usize + 1);

        if self.cells_bounds.y > *required.start() {
            let new_y = self.row_heights.sum_to(row as usize);
            let row_heights = self.row_heights.clone();
            self.set_y_offset(new_y, self.cells_bounds.height, &row_heights);
        } else if self.cells_bounds.y + self.cells_bounds.height < *required.end() {
            let first_row = self
                .row_heights
                .index_of_sum(required.end() - self.cells_bounds.height, Rounding::Up)
                .map(|i| i + 1)
                .unwrap_or(row as usize);
            let new_y = self.row_heights.sum_to(first_row);
            let row_heights = self.row_heights.clone();
            self.set_y_offset(new_y, self.cells_bounds.height, &row_heights);
        }
    }

    /// Snaps the scroll position to a [`RelativeOffset`].
    fn snap_to(&mut self, offset: RelativeOffset) {
        let column_widths = self.column_widths.clone();
        let row_heights = self.row_heights.clone();

        self.scroll_x_to(
            offset.x.clamp(0.0, 1.0),
            self.cells_bounds.width,
            &column_widths,
        );
        self.scroll_y_to(
            offset.y.clamp(0.0, 1.0),
            self.cells_bounds.height,
            &row_heights,
        );
    }

    /// Scroll to the provided [`AbsoluteOffset`].
    fn scroll_to(&mut self, offset: AbsoluteOffset) {
        let column_widths = self.column_widths.clone();
        let row_heights = self.row_heights.clone();

        self.set_x_offset(offset.x.max(0.0), self.cells_bounds.width, &column_widths);
        self.set_y_offset(offset.y.max(0.0), self.cells_bounds.height, &row_heights);
    }

    // /// Unsnaps the current scroll position, if snapped, given the context.
    // fn unsnap(&mut self, scales: ScrollScales) {
    //     self.offset_x = Offset::Absolute(self.offset_x.absolute(&scales.x()));
    //     self.offset_y = Offset::Absolute(self.offset_y.absolute(&scales.y()));
    // }

    /// Returns the scrolling offset of the [`GridScrollableState`], given the context.
    pub fn absolute_offset(&self) -> Vector {
        Vector::new(self.cells_bounds.x, self.cells_bounds.y)
    }

    pub fn is_viewport_notified(&self) -> bool {
        self.last_notified.is_some()
    }

    pub fn notify_viewport_change<Message>(
        &mut self,
        on_change: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
        viewport_size: Size,
        column_widths: &Rc<SumSeq>,
        row_heights: &Rc<SumSeq>,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.last_notified.is_none() {
            self.set_x_offset(0.0, viewport_size.width, column_widths);
            self.set_y_offset(0.0, viewport_size.height, row_heights);
        }

        if let Some(on_change) = on_change {
            let cells_width = column_widths.sum();
            let cells_height = row_heights.sum();
            let can_scroll_x = cells_width > viewport_size.width;
            let can_scroll_y = cells_height > viewport_size.height;

            if !can_scroll_x && !can_scroll_y {
                return;
            }

            let viewport = self.viewport();

            // Don't publish redundant viewports to shell
            if let Some(last_viewport) = self.last_notified {
                let unchanged =
                    |a: f32, b: f32| (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan());

                if unchanged(last_viewport.absolute.x, viewport.absolute.x)
                    && unchanged(last_viewport.absolute.y, viewport.absolute.y)
                    && unchanged(last_viewport.relative.x, viewport.relative.x)
                    && unchanged(last_viewport.relative.y, viewport.relative.y)
                    && last_viewport.range != viewport.range
                {
                    return;
                }
            }

            shell.publish(on_change(viewport));
            self.last_notified = Some(viewport);
        }
    }
}

impl Default for GridScrollableState {
    fn default() -> Self {
        Self {
            horizontal_granularity: Default::default(),
            vertical_granularity: Default::default(),

            scroll_area_touched_at: None,
            y_scroller_grabbed_at: None,
            x_scroller_grabbed_at: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            unused_x_delta: 0.0,
            unused_y_delta: 0.0,
            unused_delta_last_updated: Instant::now(),

            cells_bounds: Rectangle::new(Point::new(-1.0, -1.0), Size::ZERO),
            column_widths: Rc::new(SumSeq::new()),
            row_heights: Rc::new(SumSeq::new()),
            last_notified: None,
        }
    }
}

impl iced::advanced::widget::operation::Scrollable for GridScrollableState {
    fn snap_to(&mut self, offset: RelativeOffset) {
        GridScrollableState::snap_to(self, offset);
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset) {
        GridScrollableState::scroll_to(self, offset)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Granularity {
    #[default]
    Discrete,
    Continuous,
}

fn quantize(
    value: f32,
    viewport_size: f32,
    discretes: Rc<SumSeq>,
    granularity: Granularity,
) -> f32 {
    let value = value.clamp(0.0, (discretes.sum() - viewport_size).max(0.0));
    if granularity == Granularity::Discrete {
        let index = discretes.index_of_sum(value, Rounding::Down).unwrap_or(0);
        let start = discretes.sum_to(index);
        let end = discretes.sum_to(index + 1);
        if value > (start + end) / 2.0 {
            end
        } else {
            start
        }
    } else {
        value
    }
}

struct GridPartsBounds {
    row_heads: Option<Rectangle>,
    column_heads: Option<Rectangle>,
    corner: Option<Rectangle>,
    cells: Rectangle,
}

impl GridPartsBounds {
    fn new(tree: &Tree, layout: Layout<'_>) -> Self {
        let mut row_heads = None;
        let mut column_heads = None;
        let mut corner = None;
        let mut cells = None;

        for (t, l) in tree.children.iter().zip(layout.children()) {
            if t.tag == tree::Tag::of::<crate::grid::RowHeadsState>() {
                row_heads = Some(l.bounds());
            } else if t.tag == tree::Tag::of::<crate::grid::ColumnHeadsState>() {
                column_heads = Some(l.bounds());
            } else if t.tag == tree::Tag::of::<crate::grid::CornerState>() {
                corner = Some(l.bounds());
            } else if t.tag == tree::Tag::of::<crate::grid::GridCellsState>() {
                cells = Some(l.bounds());
            }
        }

        Self {
            row_heads,
            column_heads,
            corner,
            cells: cells.expect("Cells must always be present"),
        }
    }

    fn row_heads_width(&self) -> f32 {
        self.row_heads.map_or(0.0, |b| b.width)
    }

    fn column_heads_height(&self) -> f32 {
        self.column_heads.map_or(0.0, |b| b.height)
    }
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
pub struct ScrollableParts {
    pub y_scrollbar: Option<GridScrollbar>,
    pub x_scrollbar: Option<GridScrollbar>,
    pub cells_viewport: Rectangle,
    pub row_heads_viewport: Option<Rectangle>,
    pub column_heads_viewport: Option<Rectangle>,
    pub corner_viewport: Option<Rectangle>,
}

impl ScrollableParts {
    pub fn full_bounds(&self) -> Rectangle {
        let x = self
            .row_heads_viewport
            .map(|v| v.x)
            .unwrap_or(self.cells_viewport.x);
        let y = self
            .column_heads_viewport
            .map(|v| v.y)
            .unwrap_or(self.cells_viewport.y);

        let width = self.cells_viewport.width
            + self.row_heads_viewport.map(|v| v.width).unwrap_or(0.0)
            + self
                .y_scrollbar
                .map(|sb| sb.total_bounds.width)
                .unwrap_or(0.0);
        let height = self.cells_viewport.height
            + self.column_heads_viewport.map(|v| v.height).unwrap_or(0.0)
            + self
                .x_scrollbar
                .map(|sb| sb.total_bounds.height)
                .unwrap_or(0.0);

        Rectangle::new(Point::new(x, y), Size::new(width, height))
    }

    pub fn is_mouse_over_x_scrollbar(&self, cursor: mouse::Cursor) -> bool {
        if let (Some(cursor_position), Some(scrollbar)) = (cursor.position(), self.x_scrollbar) {
            scrollbar.is_mouse_over(cursor_position)
        } else {
            false
        }
    }

    pub fn is_mouse_over_y_scrollbar(&self, cursor: mouse::Cursor) -> bool {
        if let (Some(cursor_position), Some(scrollbar)) = (cursor.position(), self.y_scrollbar) {
            scrollbar.is_mouse_over(cursor_position)
        } else {
            false
        }
    }

    pub fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.y_scrollbar.and_then(|scrollbar| {
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

    pub fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.x_scrollbar.and_then(|scrollbar| {
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

    pub fn can_scroll(&self) -> bool {
        self.y_scrollbar.is_some() || self.x_scrollbar.is_some()
    }

    pub fn fill_ins(&self) -> impl Iterator<Item = Rectangle> + '_ {
        let mut fill_ins = vec![];

        if let (Some(x), Some(y)) = (self.x_scrollbar, self.y_scrollbar) {
            fill_ins.push(
                // Scroll bars fill in
                Rectangle::new(
                    Point::new(
                        self.cells_viewport.x + self.cells_viewport.width,
                        self.cells_viewport.y + self.cells_viewport.height,
                    ),
                    Size::new(y.total_bounds.width, x.total_bounds.height),
                ),
            );
        }

        if let (Some(xsb), Some(rhb)) = (self.x_scrollbar, self.row_heads_viewport) {
            // Row heads fill in
            fill_ins.push(Rectangle::new(
                Point::new(rhb.x, xsb.bounds.y),
                Size::new(rhb.width, xsb.bounds.height),
            ));
        }

        if let (Some(ysb), Some(chb)) = (self.y_scrollbar, self.column_heads_viewport) {
            fill_ins.push(Rectangle::new(
                Point::new(ysb.bounds.x, chb.y),
                Size::new(ysb.bounds.width, chb.height),
            ));
        };

        fill_ins.into_iter()
    }
}

/// The scrollbar of a [`GridScrollable`].
#[derive(Debug, Copy, Clone)]
pub struct GridScrollbar {
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
    pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.total_bounds.contains(cursor_position)
    }

    /// Returns the y-axis scrolled percentage from the cursor position.
    pub fn scroll_percentage_y(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
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
    pub fn scroll_percentage_x(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
        if cursor_position.x < 0.0 && cursor_position.y < 0.0 {
            (self.scroller.bounds.x / self.total_bounds.width).round()
        } else {
            (cursor_position.x - self.bounds.x - self.scroller.bounds.width * grabbed_at)
                / (self.bounds.width - self.scroller.bounds.width)
        }
    }

    pub fn draw<Renderer>(&self, renderer: &mut Renderer, style: Scrollbar)
    where
        Renderer: iced::advanced::Renderer,
        Renderer::Theme: StyleSheet,
        Renderer::Theme: crate::style::StyleSheet,
    {
        // Track
        if self.bounds.width > 0.0
            && self.bounds.height > 0.0
            && (style.background.is_some()
                || (style.border_color != Color::TRANSPARENT && style.border_width > 0.0))
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: self.bounds,
                    border_radius: style.border_radius,
                    border_width: style.border_width,
                    border_color: style.border_color,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        // Thumb
        if self.scroller.bounds.width > 0.0
            && self.scroller.bounds.height > 0.0
            && (style.scroller.color != Color::TRANSPARENT
                || (style.scroller.border_color != Color::TRANSPARENT
                    && style.scroller.border_width > 0.0))
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: self.scroller.bounds,
                    border_radius: style.scroller.border_radius,
                    border_width: style.scroller.border_width,
                    border_color: style.scroller.border_color,
                },
                style.scroller.color,
            );
        }
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
pub struct GridScroller {
    /// The bounds of the [`Scroller`].
    bounds: Rectangle,
}
