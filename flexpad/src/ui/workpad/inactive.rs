use flexpad_grid::RowCol;
use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse,
        renderer::Style,
        text,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    event::Status,
    touch,
    widget::{text::LineHeight, text_input::Value},
    Color, Element, Event, Length, Pixels, Rectangle, Size,
};

use super::WorkpadMessage;

pub struct InactiveCell<Renderer>
where
    Renderer: text::Renderer,
{
    rc: RowCol,
    value: Value,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    font_size: f32,
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
            font_size: 10.0,
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
        self.font_size = size.into().0;
        self
    }
}

impl<Renderer> Widget<WorkpadMessage, Renderer> for InactiveCell<Renderer>
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

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
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

        let text_width = renderer.measure_width(&text, size, font, text::Shaping::Advanced);

        let render = |renderer: &mut Renderer| {
            // TODO Colors
            let color = Color::BLACK;
            let h_align = self.horizontal_alignment;
            let v_align = self.vertical_alignment;
            let width = bounds.width;
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
            let text_bounds = Rectangle {
                x,
                y,
                width,
                ..bounds
            };
            renderer.fill_text(Text {
                content: &text,
                color,
                font,
                bounds: text_bounds,
                size,
                line_height: LineHeight::default(),
                horizontal_alignment: h_align,
                vertical_alignment: v_align,
                shaping: text::Shaping::Advanced,
            });
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
        _operation: &mut dyn Operation<WorkpadMessage>,
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
        shell: &mut Shell<'_, WorkpadMessage>,
        _viewport: &Rectangle,
    ) -> Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.position_over(layout.bounds()).is_some() {
                    shell.publish(WorkpadMessage::ActiveCellMove(super::Move::To(self.rc)));
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            _ => Status::Ignored,
        }
    }
}

impl<'a, Renderer> From<InactiveCell<Renderer>> for Element<'a, WorkpadMessage, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer: text::Renderer,
{
    fn from(cell: InactiveCell<Renderer>) -> Self {
        Self::new(cell)
    }
}
