use iced::advanced::layout::Limits;
use iced::advanced::overlay::Group;
use iced::advanced::widget::tree::Tree;
use iced::advanced::widget::Operation;
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::{Cursor, Interaction};
use iced::{event, Color, Element, Event, Length, Point, Rectangle, Size, Vector};
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::rc::Rc;

use crate::{ColumnHead, GridCell, GridCorner, RowCol, RowHead, SumSeq};

pub mod addressing;
pub mod cell;
pub mod head;
pub mod operation;
pub mod scroll;
pub mod style;

use head::{ColumnHeads, RowHeads};
pub use style::{Appearance, StyleSheet};

/// A container that distributes its contents as a grid.
pub struct Grid<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    row_heights: Rc<SumSeq>,
    column_widths: Rc<SumSeq>,
    width: Length,
    height: Length,
    cells: Vec<GridCell<'a, Message, Renderer>>,
    row_heads: Option<RowHeads<'a, Message, Renderer>>,
    column_heads: Option<ColumnHeads<'a, Message, Renderer>>,
    corner: Option<GridCorner<'a, Message, Renderer>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Grid<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    <Renderer::Theme as StyleSheet>::Style: Clone,
{
    /// Creates an empty [`Grid`].
    pub fn new(row_heights: SumSeq, column_widths: SumSeq) -> Self {
        Grid {
            row_heights: Rc::new(row_heights),
            column_widths: Rc::new(column_widths),
            width: Length::Shrink,
            height: Length::Shrink,
            cells: vec![],
            row_heads: None,
            column_heads: None,
            corner: None,
            style: Default::default(),
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
        // TODO check for existing cells that this overlaps and remove them
        self.cells.push(cell);
        self
    }

    /// Adds a [`RowHead`] element to the [`Grid`].
    pub fn push_row_head(mut self, head: RowHead<'a, Message, Renderer>) -> Self {
        let rh = match self.row_heads {
            Some(rh) => rh,
            None => RowHeads::new(Rc::clone(&self.row_heights)),
        };
        let rh = rh.push(head.head);
        let rh = rh.style(self.style.clone());
        self.row_heads = Some(rh);
        self
    }

    /// Sets the width of the row headings for the [`Grid`].
    pub fn row_head_width(mut self, width: impl Into<Length>) -> Self {
        let rh = match self.row_heads {
            Some(rh) => rh,
            None => RowHeads::new(self.row_heights.clone()),
        };
        let rh = rh.width(width.into());
        let rh = rh.style(self.style.clone());
        self.row_heads = Some(rh);
        self
    }

    /// Adds a [`ColumnHead`] element to the [`Grid`].
    pub fn push_column_head(mut self, head: ColumnHead<'a, Message, Renderer>) -> Self {
        let ch = match self.column_heads {
            Some(ch) => ch,
            None => ColumnHeads::new(Rc::clone(&self.column_widths)),
        };
        let ch = ch.push(head.head);
        let ch = ch.style(self.style.clone());
        self.column_heads = Some(ch);
        self
    }

    /// Sets the height of the column headings for the [`Grid`].
    pub fn column_head_height(mut self, height: impl Into<Length>) -> Self {
        let ch = match self.column_heads {
            Some(ch) => ch,
            None => ColumnHeads::new(self.column_widths.clone()),
        };
        let ch = ch.height(height.into());
        let ch = ch.style(self.style.clone());
        self.column_heads = Some(ch);
        self
    }

    /// Adds a [`GridCorner`] element to the [`Grid`].  Note that the corner is only visible
    /// where both row and column heads are used.
    pub fn push_corner(mut self, head: GridCorner<'a, Message, Renderer>) -> Self {
        self.corner = Some(head);
        self
    }

    /// Sets the style of the [`Grid`].
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();

        if let Some(rh) = self.row_heads {
            let rh = rh.style(self.style.clone());
            self.row_heads = Some(rh);
        }

        if let Some(ch) = self.column_heads {
            let ch = ch.style(self.style.clone());
            self.column_heads = Some(ch);
        }

        self
    }

    fn draw_background(&self, bounds: Rectangle, renderer: &mut Renderer, theme: &Renderer::Theme) {
        let appearance = theme.appearance(&self.style);

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
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Grid<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    <Renderer::Theme as StyleSheet>::Style: Clone,
{
    fn children(&self) -> Vec<Tree> {
        let mut result = vec![];
        if let Some(ref widget) = self.row_heads {
            result.push(Tree::new(widget));
        }
        if let Some(ref widget) = self.column_heads {
            result.push(Tree::new(widget));
        }
        if self.row_heads.is_some() && self.column_heads.is_some() {
            if let Some(ref widget) = self.corner {
                result.push(Tree::new(&widget.head));
            }
        }
        result.extend(self.cells.iter().map(Tree::new));
        result
    }

    fn diff(&self, tree: &mut Tree) {
        let new_children_len = self.cells.len() + if self.column_heads.is_some() { 1 } else { 0 };
        if tree.children.len() > new_children_len {
            tree.children.truncate(new_children_len);
        }

        let mut i = 0;

        if let Some(ref widget) = self.row_heads {
            if i < tree.children.len() {
                tree.children[i].diff(widget)
            } else {
                tree.children.push(Tree::new(widget))
            }
            i += 1;
        }

        if let Some(ref widget) = self.column_heads {
            if i < tree.children.len() {
                tree.children[i].diff(widget)
            } else {
                tree.children.push(Tree::new(widget))
            }
            i += 1;
        }

        if self.row_heads.is_some() && self.column_heads.is_some() {
            if let Some(ref widget) = self.corner {
                if i < tree.children.len() {
                    tree.children[i].diff(&widget.head)
                } else {
                    tree.children.push(Tree::new(&widget.head))
                }
                i += 1;
            }
        }

        for cell in self.cells.iter() {
            if i < tree.children.len() {
                tree.children[i].diff(cell)
            } else {
                tree.children.push(Tree::new(cell))
            }
            i += 1;
        }
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let width = self.column_widths.sum();
        let height = self.row_heights.sum();

        let mut children = vec![];

        let r_heads_layout = self
            .row_heads
            .as_ref()
            .map(|ch| ch.layout(renderer, &limits.loose()));
        let c_heads_layout = self
            .column_heads
            .as_ref()
            .map(|ch| ch.layout(renderer, &limits.loose()));

        let heads_offset = match (r_heads_layout, c_heads_layout) {
            (None, None) => Vector::new(0.0, 0.0),
            (None, Some(ch_layout)) => {
                let result = Vector::new(0.0, ch_layout.size().height);
                children.push(ch_layout);
                result
            }
            (Some(rh_layout), None) => {
                let result = Vector::new(rh_layout.size().width, 0.0);
                children.push(rh_layout);
                result
            }
            (Some(mut rh_layout), Some(mut ch_layout)) => {
                let result = Vector::new(rh_layout.size().width, ch_layout.size().height);
                rh_layout.move_to(Point::new(0.0, result.y));
                ch_layout.move_to(Point::new(result.x, 0.0));
                children.push(rh_layout);
                children.push(ch_layout);

                // Corner only used when row and column heads are used
                if let Some(ref widget) = self.corner {
                    let corner_limits = limits.loose().max_width(result.x).max_height(result.y);
                    let corner_layout = widget.head.layout(renderer, &corner_limits);
                    children.push(corner_layout);
                }

                result
            }
        };

        for child_cell in self.cells.iter() {
            let rows = child_cell.range.rows();
            let y1 = self.row_heights.sum_to(rows.start as usize);
            let y2 = self.row_heights.sum_to(rows.end as usize);
            let columns = child_cell.range.columns();
            let x1 = self.column_widths.sum_to(columns.start as usize);
            let x2 = self.column_widths.sum_to(columns.end as usize);
            let cell_size = Size::new(x2 - x1, y2 - y1);
            let cell_limits = Limits::new(cell_size, cell_size);
            let mut child_layout = child_cell.layout(renderer, &cell_limits);
            child_layout.move_to(Point::new(x1, y1) + heads_offset);
            children.push(child_layout);
        }

        layout::Node::with_children(
            Size::new(width + heads_offset.x, height + heads_offset.y),
            children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            let mut child_trees = tree.children.iter_mut();
            let mut child_layouts = layout.children();

            if let Some(ref r_heads) = self.row_heads {
                r_heads.operate(
                    child_trees.next().unwrap(),
                    child_layouts.next().unwrap(),
                    renderer,
                    operation,
                );
            };

            if let Some(ref c_heads) = self.column_heads {
                c_heads.operate(
                    child_trees.next().unwrap(),
                    child_layouts.next().unwrap(),
                    renderer,
                    operation,
                );
            };

            if self.row_heads.is_some() && self.column_heads.is_some() {
                if let Some(ref widget) = self.corner {
                    widget.head.operate(
                        child_trees.next().unwrap(),
                        child_layouts.next().unwrap(),
                        renderer,
                        operation,
                    );
                }
            }

            self.cells
                .iter()
                .zip(child_trees)
                .zip(child_layouts)
                .for_each(|((cell, tree), layout)| {
                    cell.operate(tree, layout, renderer, operation);
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
        let mut child_trees = tree.children.iter_mut();
        let mut child_layouts = layout.children();
        let mut status = event::Status::Ignored;

        if let Some(ref mut r_heads) = self.row_heads {
            let s = r_heads.on_event(
                child_trees.next().unwrap(),
                event.clone(),
                child_layouts.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            status = status.merge(s);
        };

        if let Some(ref mut c_heads) = self.column_heads {
            let s = c_heads.on_event(
                child_trees.next().unwrap(),
                event.clone(),
                child_layouts.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            status = status.merge(s);
        };

        if self.row_heads.is_some() && self.column_heads.is_some() {
            if let Some(ref mut widget) = self.corner {
                let s = widget.head.on_event(
                    child_trees.next().unwrap(),
                    event.clone(),
                    child_layouts.next().unwrap(),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
                status = status.merge(s);
            }
        }

        self.cells
            .iter_mut()
            .zip(child_trees)
            .zip(child_layouts)
            .map(|((child, tree), layout)| {
                child.on_event(
                    tree,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(status, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut child_trees = tree.children.iter();
        let mut child_layouts = layout.children();
        let mut result = Interaction::default();

        if let Some(ref r_heads) = self.row_heads {
            let i = r_heads.mouse_interaction(
                child_trees.next().unwrap(),
                child_layouts.next().unwrap(),
                cursor,
                viewport,
                renderer,
            );
            result = result.max(i);
        };

        if let Some(ref c_heads) = self.column_heads {
            let i = c_heads.mouse_interaction(
                child_trees.next().unwrap(),
                child_layouts.next().unwrap(),
                cursor,
                viewport,
                renderer,
            );
            result = result.max(i);
        };

        if self.row_heads.is_some() && self.column_heads.is_some() {
            if let Some(ref widget) = self.corner {
                let i = widget.head.mouse_interaction(
                    child_trees.next().unwrap(),
                    child_layouts.next().unwrap(),
                    cursor,
                    viewport,
                    renderer,
                );
                result = result.max(i);
            }
        }

        self.cells
            .iter()
            .zip(child_trees)
            .zip(child_layouts)
            .map(|((child, tree), layout)| {
                child.mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .fold(result, Interaction::max)
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
        let mut child_trees = tree.children.iter();
        let mut child_layouts = layout.children();

        let appearance = theme.appearance(&self.style);
        let bounds = layout.bounds();

        self.draw_background(bounds, renderer, theme);

        // Row Header
        let mut r_heads_width = 0.0;
        if let Some(ref r_heads) = self.row_heads {
            let r_heads_tree = child_trees.next().unwrap();
            let r_heads_layout = child_layouts.next().unwrap();
            r_heads_width = r_heads_layout.bounds().width;

            r_heads.draw(
                r_heads_tree,
                renderer,
                theme,
                renderer_style,
                r_heads_layout,
                cursor,
                viewport,
            );
        };

        // Column Header
        let mut c_heads_height = 0.0;
        if let Some(ref c_heads) = self.column_heads {
            let c_heads_tree = child_trees.next().unwrap();
            let c_heads_layout = child_layouts.next().unwrap();
            c_heads_height = c_heads_layout.bounds().height;

            c_heads.draw(
                c_heads_tree,
                renderer,
                theme,
                renderer_style,
                c_heads_layout,
                cursor,
                viewport,
            );
        };

        // Corner
        if self.row_heads.is_some() && self.column_heads.is_some() {
            // Draw corner rule lines
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        layout.position(),
                        Size::new(r_heads_width, c_heads_height),
                    ),
                    border_radius: 0.0.into(),
                    border_width: appearance.heads_rule_width,
                    border_color: appearance.heads_rule_color,
                },
                Color::TRANSPARENT,
            );

            if let Some(ref corner) = self.corner {
                let corner_tree = child_trees.next().unwrap();
                let corner_layout = child_layouts.next().unwrap();

                corner.head.draw(
                    corner_tree,
                    renderer,
                    theme,
                    renderer_style,
                    corner_layout,
                    cursor,
                    viewport,
                );
            }
        }

        // Track which cells of the grid have been included
        // This should only cover visible ranges once scrolling is introduced
        // let mut absent_cells = BTreeSet::new();
        // for rw in 0..self.row_heights.len() {
        //     for cl in 0..self.column_widths.len() {
        //         absent_cells.insert(RowCol::new(rw as u32, cl as u32));
        //     }
        // }

        // Cells (that are present)
        for ((cell, tree), layout) in self.cells.iter().zip(child_trees).zip(child_layouts) {
            // Rule lines for this (posssible spanning) cell
            // renderer.fill_quad(
            //     renderer::Quad {
            //         bounds: layout.bounds(),
            //         border_radius: 0.0.into(),
            //         border_width: appearance.rule_width,
            //         border_color: appearance.rule_color,
            //     },
            //     Color::TRANSPARENT,
            // );

            // cell.range.cells().for_each(|rc| {
            //     absent_cells.remove(&rc);
            // });

            cell.draw(
                tree,
                renderer,
                theme,
                renderer_style,
                layout,
                cursor,
                viewport,
            );
        }

        // Draw rule lines for the absent cells
        // let heads_offset = Vector::new(r_heads_width, c_heads_height);
        // for absent_cell in absent_cells {
        //     let rows = absent_cell.rows();
        //     let y1 = self.row_heights.sum_to(rows.start as usize);
        //     let y2 = self.row_heights.sum_to(rows.end as usize);
        //     let columns = absent_cell.columns();
        //     let x1 = self.column_widths.sum_to(columns.start as usize);
        //     let x2 = self.column_widths.sum_to(columns.end as usize);
        //     let cell_bounds = Rectangle::new(
        //         bounds.position() + Vector::new(x1, y1) + heads_offset,
        //         Size::new(x2 - x1, y2 - y1),
        //     );
        //     renderer.fill_quad(
        //         renderer::Quad {
        //             bounds: cell_bounds,
        //             border_radius: 0.0.into(),
        //             border_width: appearance.rule_width,
        //             border_color: appearance.rule_color,
        //         },
        //         Color::TRANSPARENT,
        //     );
        // }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let mut children = vec![];

        let mut child_trees = tree.children.iter_mut();
        let mut child_layouts = layout.children();
        let corner_visible = self.row_heads.is_some() && self.column_heads.is_some();

        if let Some(ref mut r_heads) = self.row_heads {
            let o = r_heads.overlay(
                child_trees.next().unwrap(),
                child_layouts.next().unwrap(),
                renderer,
            );
            if let Some(o) = o {
                children.push(o);
            }
        };

        if let Some(ref mut c_heads) = self.column_heads {
            let o = c_heads.overlay(
                child_trees.next().unwrap(),
                child_layouts.next().unwrap(),
                renderer,
            );
            if let Some(o) = o {
                children.push(o);
            }
        };

        if corner_visible {
            if let Some(ref mut widget) = self.corner {
                let o = widget.head.overlay(
                    child_trees.next().unwrap(),
                    child_layouts.next().unwrap(),
                    renderer,
                );
                if let Some(o) = o {
                    children.push(o);
                }
            }
        }

        children.extend(
            self.cells
                .iter_mut()
                .zip(child_trees)
                .zip(child_layouts)
                .filter_map(|((child, tree), layout)| child.overlay(tree, layout, renderer)),
        );

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<Grid<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
    <Renderer::Theme as StyleSheet>::Style: Clone,
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
    <Renderer::Theme as StyleSheet>::Style: Clone,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}
