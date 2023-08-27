use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use iced::advanced::renderer::Quad;
use iced::advanced::widget::tree::Tree;
use iced::advanced::widget::Operation;
use iced::advanced::{layout, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::{self, Cursor};
use iced::{
    alignment, event, Alignment, Color, Element, Event, Length, Padding, Point, Rectangle, Size,
};

use super::GridInfo;
use crate::grid::style::StyleSheet;
use crate::{style, Border, Borders, CellRange};

pub struct GridCell<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub range: CellRange,
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    borders: Borders,
}

impl<'a, Message, Renderer> GridCell<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a  [`GridCell`].
    pub fn new<R, T>(range: R, content: T) -> Self
    where
        R: Into<CellRange>,
        T: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            range: range.into(),
            content: content.into(),
            padding: Padding::from(4),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            borders: Borders::NONE,
        }
    }

    /// Sets the [`Padding`] of the [`GridCell`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the [`Borders`] of the [`GridCell`].
    pub fn borders<B: Into<Borders>>(mut self, borders: B) -> Self {
        self.borders = borders.into();
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`GridCell`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`GridCell`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub(super) fn into_grid_widget(
        self,
        info: Rc<RefCell<GridInfo<Renderer>>>,
    ) -> GridCellWidget<'a, Message, Renderer> {
        GridCellWidget {
            range: self.range,
            content: self.content,
            padding: self.padding,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            borders: self.borders,
            info,
        }
    }
}

pub(crate) struct GridCellWidget<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub range: CellRange,
    content: Element<'a, Message, Renderer>,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    borders: Borders,
    info: Rc<RefCell<GridInfo<Renderer>>>,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for GridCellWidget<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .loose()
            .max_width(f32::INFINITY)
            .max_height(f32::INFINITY)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut content = self
            .content
            .as_widget()
            .layout(renderer, &limits.pad(self.padding));
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
        tree: &mut Tree,
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
        tree: &mut Tree,
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
        tree: &Tree,
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
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let info = (*self.info).borrow();
        let appearance = theme.appearance(&info.style);

        // Rule lines for this (posssible spanning) cell
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_radius: 0.0.into(),
                border_width: appearance.rule_width,
                border_color: appearance.rule_color,
            },
            Color::TRANSPARENT,
        );

        let bounds = layout.bounds();
        let mut draw_border = |bounds, color: Color| {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border_radius: 0.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                color,
            )
        };

        if self.borders.top != Border::NONE {
            draw_border(
                Rectangle::new(
                    bounds.position(),
                    Size {
                        width: bounds.width,
                        height: self.borders.top.width,
                    },
                ),
                self.borders.top.color,
            );
        }
        if self.borders.right != Border::NONE {
            draw_border(
                Rectangle::new(
                    Point::new(bounds.x + bounds.width - self.borders.right.width, bounds.y),
                    Size {
                        width: self.borders.right.width,
                        height: bounds.height,
                    },
                ),
                self.borders.right.color,
            );
        }
        if self.borders.bottom != Border::NONE {
            draw_border(
                Rectangle::new(
                    Point::new(
                        bounds.x,
                        bounds.y + bounds.height - self.borders.bottom.width,
                    ),
                    Size {
                        width: bounds.width,
                        height: self.borders.bottom.width,
                    },
                ),
                self.borders.bottom.color,
            );
        }
        if self.borders.left != Border::NONE {
            draw_border(
                Rectangle::new(
                    bounds.position(),
                    Size {
                        width: self.borders.left.width,
                        height: bounds.height,
                    },
                ),
                self.borders.left.color,
            );
        }

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
        tree: &'b mut Tree,
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

impl<'a, Message, Renderer> From<GridCellWidget<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: style::StyleSheet,
{
    fn from(cell: GridCellWidget<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(cell)
    }
}

impl<'a, Message, Renderer> Borrow<dyn Widget<Message, Renderer> + 'a>
    for &GridCellWidget<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: style::StyleSheet,
{
    fn borrow(&self) -> &(dyn Widget<Message, Renderer> + 'a) {
        *self
    }
}
