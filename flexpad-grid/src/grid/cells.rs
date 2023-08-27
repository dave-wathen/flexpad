use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use iced::{
    advanced::{
        layout::{self, Limits},
        overlay::Group,
        renderer,
        widget::{tree, Operation},
        Clipboard, Layout, Shell, Widget,
    },
    event,
    mouse::{self, Cursor},
    overlay, Element, Event, Length, Point, Rectangle, Size,
};

use super::cell::GridCellWidget;
use crate::StyleSheet;

use super::GridInfo;

// A container for the cells of a [`Grid`]
// Only used internally by Grid.
pub(super) struct GridCells<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    info: Rc<RefCell<GridInfo<Renderer>>>,
    cells: Vec<GridCellWidget<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> GridCells<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`GridCells`].
    pub fn new(info: Rc<RefCell<GridInfo<Renderer>>>) -> Self {
        Self {
            info,
            cells: vec![],
        }
    }

    /// Adds an [`RowHead`] element to the [`RowHeads`].
    pub fn push(mut self, cell: GridCellWidget<'a, Message, Renderer>) -> Self {
        // TODO check for existing cells that this overlaps and remove them
        self.cells.push(cell);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for GridCells<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<super::GridCellsState>()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.cells.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.cells.iter().collect::<Vec<_>>());
    }

    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let info = (*self.info).borrow();
        let height = info.row_heights.sum();
        let width = info.column_widths.sum();

        let children = self
            .cells
            .iter()
            .map(|cell| {
                let rows = cell.range.rows();
                let y1 = info.row_heights.sum_to(rows.start as usize);
                let y2 = info.row_heights.sum_to(rows.end as usize);
                let columns = cell.range.columns();
                let x1 = info.column_widths.sum_to(columns.start as usize);
                let x2 = info.column_widths.sum_to(columns.end as usize);
                let cell_size = Size::new(x2 - x1, y2 - y1);
                let cell_limits = Limits::new(cell_size, cell_size);
                let mut cell_layout = cell.layout(renderer, &cell_limits);
                cell_layout.move_to(Point::new(x1, y1));
                cell_layout
            })
            .collect();

        let size = limits
            .width(self.width())
            .height(self.height())
            .resolve(Size::new(width, height));
        layout::Node::with_children(size, children)
    }

    fn operate(
        &self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.cells
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.operate(state, layout, renderer, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut tree::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.cells
            .iter_mut()
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
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.cells
            .iter()
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
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self.cells.iter().zip(&tree.children).zip(layout.children())
        {
            if viewport.intersects(&layout.bounds()) {
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
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let children = self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| child.overlay(state, layout, renderer))
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<GridCells<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(grid_cells: GridCells<'a, Message, Renderer>) -> Self {
        Self::new(grid_cells)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &GridCells<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}
