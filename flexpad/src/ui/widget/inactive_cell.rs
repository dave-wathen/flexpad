use flexpad_grid::RowCol;
use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse,
        renderer::Style,
        text::{self, Paragraph},
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    event::Status,
    touch,
    widget::{text::LineHeight, text_input::Value},
    Color, Element, Event, Length, Pixels, Point, Rectangle, Size,
};

use crate::ui::active_sheet::{Message, Move};

pub struct InactiveCell<Renderer>
where
    Renderer: text::Renderer,
{
    rc: RowCol,
    value: Value,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    font_size: Pixels,
}

impl<Renderer> InactiveCell<Renderer>
where
    Renderer: text::Renderer,
{
    /// Creates a new [`InactiveCell`].
    pub fn new(rc: RowCol, value: &str) -> Self {
        Self {
            rc,
            value: Value::new(value),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            font: None,
            font_size: Pixels::from(10.0),
        }
    }

    /// Sets the horizontal alignment of the [`ActiveCell`].
    pub fn horizontal_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the vertical alignment of the [`ActiveCell`].
    pub fn vertical_alignment(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Sets the [`Font`] of the [`ActiveCell`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the font size of the [`ActiveCell`].
    pub fn font_size(mut self, size: impl Into<Pixels>) -> Self {
        self.font_size = size.into();
        self
    }
}

impl<Renderer> Widget<Message, Renderer> for InactiveCell<Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer: text::Renderer,
{
    fn width(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn height(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        let bounds = limits
            .width(Length::Fill)
            .height(Length::Fill)
            .resolve(Size::ZERO);
        Node::new(bounds)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &<Renderer as iced::advanced::Renderer>::Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let text = self.value.to_string();
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let size = self.font_size;

        let paragraph = Renderer::Paragraph::with_text(Text {
            content: &text,
            bounds: Size::INFINITY,
            size,
            line_height: LineHeight::Absolute(size),
            font,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
        });
        let text_width = paragraph.min_width();

        let render = |renderer: &mut Renderer| {
            // TODO Colors
            let color = Color::BLACK;
            let h_align = self.horizontal_alignment;
            let v_align = self.vertical_alignment;
            let x = match h_align {
                alignment::Horizontal::Left => bounds.x,
                alignment::Horizontal::Center => bounds.center_x(),
                alignment::Horizontal::Right => bounds.x + bounds.width,
            };
            let y = match v_align {
                alignment::Vertical::Top => bounds.y,
                alignment::Vertical::Center => bounds.center_y(),
                alignment::Vertical::Bottom => bounds.y + bounds.width,
            };
            let text_position = Point::new(x, y);
            renderer.fill_text(
                Text {
                    content: &text,
                    font,
                    bounds: bounds.size(),
                    size,
                    line_height: LineHeight::default(),
                    horizontal_alignment: h_align,
                    vertical_alignment: v_align,
                    shaping: text::Shaping::Advanced,
                },
                text_position,
                color,
                bounds,
            );
        };

        if text_width > bounds.width {
            renderer.with_layer(bounds, |renderer| {
                // TODO Overflow rules
                render(renderer);
            });
        } else {
            render(renderer);
        }
    }

    fn operate(
        &self,
        _tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn Operation<Message>,
    ) {
        // let state = tree.state.downcast_mut::<State>();

        // operation.focusable(state, self.id.as_ref().map(|id| &id.0));
        // operation.text_input(state, self.id.as_ref().map(|id| &id.0));
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.position_over(layout.bounds()).is_some() {
                    shell.publish(Message::ActiveCellMove(Move::To(self.rc)));
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            _ => Status::Ignored,
        }
    }
}

impl<'a, Renderer> From<InactiveCell<Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer: text::Renderer,
{
    fn from(cell: InactiveCell<Renderer>) -> Self {
        Self::new(cell)
    }
}
