use iced::advanced::overlay::Group;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::Operation;
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::Cursor;
use iced::{event, Color, Element, Event, Length, Point, Rectangle, Size, Vector};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::iter::{empty, once};
use std::rc::Rc;

use crate::{ColumnHead, GridCell, GridCorner, RowHead, SumSeq};
// TODO Check which of these need to be public!
pub mod addressing;
pub mod cell;
mod cells;
pub mod head;
pub mod scroll;
mod state;
pub mod style;

use head::{ColumnHeads, Head, RowHeads};
use state::GridState;
pub use style::{Appearance, StyleSheet};

use cells::GridCells;

/// A container that distributes its contents as a grid.
pub struct Grid<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    cells: GridCells<'a, Message, Renderer>,
    row_heads: Option<RowHeads<'a, Message, Renderer>>,
    column_heads: Option<ColumnHeads<'a, Message, Renderer>>,
    corner: Option<Head<'a, Message, Renderer>>,
    info: Rc<RefCell<GridInfo<Renderer>>>,
}

#[allow(suspicious_double_ref_op)]
impl<'a, Message, Renderer> Grid<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`Grid`].
    pub fn new(row_heights: SumSeq, column_widths: SumSeq) -> Self {
        let info = GridInfo {
            row_heights: Rc::new(row_heights),
            column_widths: Rc::new(column_widths),
            style: Default::default(),
        };
        let info = Rc::new(RefCell::new(info));

        Grid {
            width: Length::Shrink,
            height: Length::Shrink,
            cells: GridCells::new(Rc::clone(&info)),
            row_heads: None,
            column_heads: None,
            corner: None,
            info: Rc::clone(&info),
        }
    }

    /// Sets the width of the [`Grid`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Grid`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Adds an [`GridCell`] element to the [`Grid`].
    pub fn push_cell(mut self, cell: GridCell<'a, Message, Renderer>) -> Self {
        let info = Rc::clone(&self.info);
        self.cells = self.cells.push(cell.into_grid_widget(info));
        self
    }

    /// Adds a [`RowHead`] element to the [`Grid`].
    pub fn push_row_head(mut self, row_head: RowHead<'a, Message, Renderer>) -> Self {
        let rh = match self.row_heads {
            Some(rh) => rh,
            None => RowHeads::new(Rc::clone(&self.info)),
        };
        let info = Rc::clone(&self.info);
        let rh = rh.push(row_head.into_grid_widget(info));
        self.row_heads = Some(rh);
        self
    }

    /// Sets the width of the row headings for the [`Grid`].
    pub fn row_head_width(mut self, width: impl Into<Length>) -> Self {
        let rh = match self.row_heads {
            Some(rh) => rh,
            None => RowHeads::new(Rc::clone(&self.info)),
        };
        let rh = rh.width(width.into());
        self.row_heads = Some(rh);
        self
    }

    /// Adds a [`ColumnHead`] element to the [`Grid`].
    pub fn push_column_head(mut self, column_head: ColumnHead<'a, Message, Renderer>) -> Self {
        let ch = match self.column_heads {
            Some(ch) => ch,
            None => ColumnHeads::new(Rc::clone(&self.info)),
        };
        let info = Rc::clone(&self.info);
        let ch = ch.push(column_head.into_grid_widget(info));
        self.column_heads = Some(ch);
        self
    }

    /// Sets the height of the column headings for the [`Grid`].
    pub fn column_head_height(mut self, height: impl Into<Length>) -> Self {
        let ch = match self.column_heads {
            Some(ch) => ch,
            None => ColumnHeads::new(Rc::clone(&self.info)),
        };
        let ch = ch.height(height.into());
        self.column_heads = Some(ch);
        self
    }

    /// Adds a [`GridCorner`] element to the [`Grid`].  Note that the corner is only visible
    /// where both row and column heads are used.
    pub fn push_corner(mut self, corner: GridCorner<'a, Message, Renderer>) -> Self {
        let info = Rc::clone(&self.info);
        self.corner = Some(corner.into_grid_widget(info));
        self
    }

    /// Sets the style of the [`Grid`].
    pub fn style(self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        {
            let mut info = (*self.info).borrow_mut();
            info.style = style.into();
        }
        self
    }

    fn widgets(&self) -> impl Iterator<Item = &dyn Widget<Message, Renderer>> {
        let corner_active = self.row_heads.is_some() && self.column_heads.is_some();

        let mut widgets = vec![];
        if let Some(ref rh) = self.row_heads {
            widgets.push(rh as &dyn Widget<Message, Renderer>);
        }
        if let Some(ref ch) = self.column_heads {
            widgets.push(ch as &dyn Widget<Message, Renderer>);
        }
        if corner_active {
            if let Some(ref c) = self.corner {
                widgets.push(c as &dyn Widget<Message, Renderer>);
            }
        }
        widgets.push(&self.cells as &dyn Widget<Message, Renderer>);
        widgets.into_iter()
    }

    fn widgets_mut(&mut self) -> impl Iterator<Item = &mut dyn Widget<Message, Renderer>> {
        let corner_active = self.row_heads.is_some() && self.column_heads.is_some();

        let mut widgets = vec![];
        if let Some(ref mut rh) = self.row_heads {
            widgets.push(rh as &mut dyn Widget<Message, Renderer>);
        }
        if let Some(ref mut ch) = self.column_heads {
            widgets.push(ch as &mut dyn Widget<Message, Renderer>);
        }
        if corner_active {
            if let Some(ref mut c) = self.corner {
                widgets.push(c as &mut dyn Widget<Message, Renderer>);
            }
        }
        widgets.push(&mut self.cells as &mut dyn Widget<Message, Renderer>);
        widgets.into_iter()
    }
}

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer> for Grid<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GridState>()
    }

    fn state(&self) -> tree::State {
        let info = (*self.info).borrow();
        tree::State::new(GridState::new(
            info.row_heights.clone(),
            info.column_widths.clone(),
        ))
    }

    fn children(&self) -> Vec<Tree> {
        let mut result = vec![];
        if let Some(ref widget) = self.row_heads {
            result.push(Tree::new(widget));
        }
        if let Some(ref widget) = self.column_heads {
            result.push(Tree::new(widget));
        }
        if self.row_heads.is_some() && self.column_heads.is_some() {
            if let Some(ref head) = self.corner {
                result.push(Tree::new(head));
            }
        }
        result.push(Tree::new(&self.cells));
        result
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.widgets().collect::<Vec<_>>());
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let mut row_heads_layout = self
            .row_heads
            .as_ref()
            .map(|ch| ch.layout(renderer, &limits.loose()));

        let mut column_heads_layout = self
            .column_heads
            .as_ref()
            .map(|ch| ch.layout(renderer, &limits.loose()));

        let (heads_offset, corner_layout) =
            match (row_heads_layout.as_mut(), column_heads_layout.as_mut()) {
                (None, None) => (Vector::new(0.0, 0.0), None),
                (None, Some(ch)) => (Vector::new(0.0, ch.size().height), None),
                (Some(rh), None) => (Vector::new(rh.size().width, 0.0), None),
                (Some(rh), Some(ch)) => {
                    let x = rh.size().width;
                    let y = ch.size().height;
                    rh.move_to(Point::new(0.0, y));
                    ch.move_to(Point::new(x, 0.0));
                    (
                        Vector::new(x, y),
                        self.corner.as_ref().map(|cnr| {
                            let corner_limits = limits.loose().max_width(x).max_height(y);
                            cnr.layout(renderer, &corner_limits)
                        }),
                    )
                }
            };

        let cell_limits = limits.loose().shrink(heads_offset.into());
        let mut cells_layout = self.cells.layout(renderer, &cell_limits);
        cells_layout.move_to(Point::ORIGIN + heads_offset);

        let gp_layout = GridPartsLayout {
            row_heads: row_heads_layout,
            column_heads: column_heads_layout,
            corner: corner_layout,
            cells: cells_layout,
        };

        gp_layout.into_layout()
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.widgets()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.operate(state, layout, renderer, operation);
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
        self.widgets_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.on_event(
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
        self.widgets()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let info = (*self.info).borrow();
        let appearance = theme.appearance(&info.style);

        // Background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            appearance
                .background
                .unwrap_or(iced::Background::Color(Color::TRANSPARENT)),
        );

        for ((child, state), layout) in self.widgets().zip(&tree.children).zip(layout.children()) {
            child.draw(
                state,
                renderer,
                theme,
                renderer_style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let children = self
            .widgets_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| child.overlay(state, layout, renderer))
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<Grid<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(grid: Grid<'a, Message, Renderer>) -> Self {
        Self::new(grid)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &Grid<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}

struct GridPartsLayout {
    row_heads: Option<layout::Node>,
    column_heads: Option<layout::Node>,
    corner: Option<layout::Node>,
    cells: layout::Node,
}

impl GridPartsLayout {
    fn into_layout(self) -> layout::Node {
        let size =
            |opt: &Option<layout::Node>| opt.as_ref().map(|l| l.size()).unwrap_or(Size::ZERO);

        let width = size(&self.row_heads).width + self.cells.size().width;
        let height = size(&self.column_heads).height + self.cells.size().height;

        let children: Vec<layout::Node> = empty()
            .chain(self.row_heads.iter())
            .chain(self.column_heads.iter())
            .chain(self.corner.iter())
            .chain(once(&self.cells))
            .cloned()
            .collect();

        layout::Node::with_children(Size::new(width, height), children)
    }
}

pub struct GridInfo<Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    row_heights: Rc<SumSeq>,
    column_widths: Rc<SumSeq>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

struct GridCellsState;
struct RowHeadsState;
struct ColumnHeadsState;
struct CornerState;
