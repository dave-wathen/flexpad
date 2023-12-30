use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use iced::advanced::overlay::Group;
use iced::advanced::widget::tree;
use iced::advanced::widget::Operation;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::{self, Cursor};
use iced::{
    alignment, event, Alignment, Color, Element, Event, Length, Padding, Point, Rectangle, Size,
};

use crate::StyleSheet;

use super::GridInfo;

// A heading for a row in a [`Grid`]
pub struct RowHead<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    row: usize,
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

#[allow(dead_code)]
impl<'a, Message, Renderer> RowHead<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a  [`RowHead`].
    pub fn new<T>(row: usize, content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            row,
            content: content.into(),
            padding: Padding::from(4),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
        }
    }

    /// Sets the [`Padding`] around the contents of the [`RowHead`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`RowHead`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`RowHead`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub(super) fn into_grid_widget(
        self,
        info: Rc<RefCell<GridInfo<Renderer>>>,
    ) -> Head<'a, Message, Renderer> {
        Head {
            index: self.row,
            content: self.content,
            padding: self.padding,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            info,
            tag: None,
        }
    }
}

// A heading for a column in a [`Grid`]
pub struct ColumnHead<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    column: usize,
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

#[allow(dead_code)]
impl<'a, Message, Renderer> ColumnHead<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a  [`ColumnHead`].
    pub fn new<T>(column: usize, content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            column,
            content: content.into(),
            padding: Padding::from(4),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
        }
    }

    /// Sets the [`Padding`] around the contents of the [`ColumnHead`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`ColumnHead`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`ColumnHead`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub(super) fn into_grid_widget(
        self,
        info: Rc<RefCell<GridInfo<Renderer>>>,
    ) -> Head<'a, Message, Renderer> {
        Head {
            index: self.column,
            content: self.content,
            padding: self.padding,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            info,
            tag: None,
        }
    }
}

// Used to place widgets in the top-left corner of a [`Grid`] when
// both column and row headers are used.
pub struct GridCorner<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

#[allow(dead_code)]
impl<'a, Message, Renderer> GridCorner<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a  [`GridCorner`].
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            content: content.into(),
            padding: Padding::from(4),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
        }
    }

    /// Sets the [`Padding`] around the contents of the [`GridCorner`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`GridCorner`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`GridCorner`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub(super) fn into_grid_widget(
        self,
        info: Rc<RefCell<GridInfo<Renderer>>>,
    ) -> Head<'a, Message, Renderer> {
        Head {
            index: 0,
            content: self.content,
            padding: self.padding,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            info,
            tag: Some(tree::Tag::of::<super::CornerState>()),
        }
    }
}

pub struct Head<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub index: usize,
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    info: Rc<RefCell<GridInfo<Renderer>>>,
    tag: Option<tree::Tag>,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Head<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        self.tag.as_ref().cloned().unwrap_or(tree::Tag::stateless())
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .loose()
            .max_width(f32::INFINITY)
            .max_height(f32::INFINITY)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut content = self.content.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &limits.pad(self.padding),
        );
        let padding = self.padding.fit(content.size(), limits.max());
        let size = limits.pad(padding).resolve(content.size());

        content.move_to(Point::new(padding.left, padding.top));
        content.align(
            Alignment::from(self.horizontal_alignment),
            Alignment::from(self.vertical_alignment),
            size,
        );

        layout::Node::with_children(size.pad(padding), vec![content])
    }

    fn operate(
        &self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut tree::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let info = (*self.info).borrow();
        let appearance = theme.appearance(&info.style);

        // Draw rule lines
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: 0.0.into(),
                border_width: appearance.heads_rule_width,
                border_color: appearance.heads_rule_color,
            },
            Color::TRANSPARENT,
        );

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: renderer_style.text_color,
            },
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Head<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(cell: Head<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(cell)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &Head<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}

// A container for the row heads of a [`Grid`]
// Only used internally by Grid.
pub(super) struct RowHeads<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    info: Rc<RefCell<GridInfo<Renderer>>>,
    width: Length,
    row_heads: Vec<Head<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> RowHeads<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`RowHeads`].
    pub fn new(info: Rc<RefCell<GridInfo<Renderer>>>) -> Self {
        Self {
            info,
            width: Length::Shrink,
            row_heads: vec![],
        }
    }

    /// Sets the width of the [`RowHeads`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Adds an [`RowHead`] element to the [`RowHeads`].
    pub fn push(mut self, head: Head<'a, Message, Renderer>) -> Self {
        self.row_heads.retain(|ch| ch.index != head.index);
        self.row_heads.push(head);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for RowHeads<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<super::RowHeadsState>()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.row_heads.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.row_heads.iter().collect::<Vec<_>>());
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let info = (*self.info).borrow();
        let height = info.row_heights.sum();
        let limits = limits.width(self.width).height(height);

        let mut children = vec![];
        let mut max_width: f32 = 0.0;
        for (r_head, tree) in self.row_heads.iter().zip(tree.children.iter_mut()) {
            let rw = r_head.index;
            let y1 = info.row_heights.sum_to(rw);
            let y2 = info.row_heights.sum_to(rw + 1);
            let cell_limits = limits.loose().max_height(y2 - y1);
            let mut child_layout = r_head.layout(tree, renderer, &cell_limits);
            max_width = max_width.max(child_layout.size().width);
            child_layout.move_to(Point::new(0.0, y1));
            children.push(child_layout);
        }

        layout::Node::with_children(limits.resolve(Size::new(max_width, height)), children)
    }

    fn operate(
        &self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.row_heads
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
        self.row_heads
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
        self.row_heads
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
        for ((child, state), layout) in self
            .row_heads
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
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
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let children = self
            .row_heads
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| child.overlay(state, layout, renderer))
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<RowHeads<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(row_heads: RowHeads<'a, Message, Renderer>) -> Self {
        Self::new(row_heads)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &RowHeads<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}

// A container for the column heads of a [`Grid`]
// Only used internally by Grid.
pub(super) struct ColumnHeads<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    info: Rc<RefCell<GridInfo<Renderer>>>,
    height: Length,
    column_heads: Vec<Head<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> ColumnHeads<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`ColumnHeads`].
    pub fn new(info: Rc<RefCell<GridInfo<Renderer>>>) -> Self {
        Self {
            info,
            height: Length::Shrink,
            column_heads: vec![],
        }
    }

    /// Sets the height of the [`ColumnHeads`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Adds an [`RowHead`] element to the [`ColumnHeads`].
    pub fn push(mut self, head: Head<'a, Message, Renderer>) -> Self {
        self.column_heads.retain(|ch| ch.index != head.index);
        self.column_heads.push(head);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ColumnHeads<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<super::ColumnHeadsState>()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.column_heads.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.column_heads.iter().collect::<Vec<_>>());
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let info = (*self.info).borrow();
        let width = info.column_widths.sum();
        let limits = limits.width(width).height(self.height);

        let mut children = vec![];
        let mut max_height: f32 = 0.0;
        for (c_head, tree) in self.column_heads.iter().zip(tree.children.iter_mut()) {
            let cl = c_head.index;
            let x1 = info.column_widths.sum_to(cl);
            let x2 = info.column_widths.sum_to(cl + 1);
            let cell_limits = limits.loose().max_width(x2 - x1);
            let mut child_layout = c_head.layout(tree, renderer, &cell_limits);
            max_height = max_height.max(child_layout.size().height);
            child_layout.move_to(Point::new(x1, 0.0));
            children.push(child_layout);
        }

        layout::Node::with_children(limits.resolve(Size::new(width, max_height)), children)
    }

    fn operate(
        &self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.column_heads
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
        self.column_heads
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
        self.column_heads
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
        for ((child, state), layout) in self
            .column_heads
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
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
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let children = self
            .column_heads
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| child.overlay(state, layout, renderer))
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<ColumnHeads<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(column_heads: ColumnHeads<'a, Message, Renderer>) -> Self {
        Self::new(column_heads)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &ColumnHeads<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}
