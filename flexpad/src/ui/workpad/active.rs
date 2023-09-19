use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer::{Quad, Style},
        text,
        widget::{self, tree, Operation, Tree},
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    event::Status,
    keyboard, mouse,
    widget::{text::LineHeight, text_input::Value},
    window, Color, Element, Event, Length, Rectangle, Size, Vector,
};

mod cursor;
mod editor;
mod platform;
pub use editor::Editor;

use super::WorkpadMessage;

/// The identifier of a [`ActiveCell`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
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

pub struct ActiveCell {
    id: Option<Id>,
    focused: bool,
    editor: Rc<RefCell<Editor>>,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font_size: f32,
}

impl ActiveCell {
    pub fn new(editor: Rc<RefCell<Editor>>) -> Self {
        Self {
            id: None,
            focused: false,
            editor,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            font_size: 10.0,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn focused(mut self, is_focused: bool) -> Self {
        self.focused = is_focused;
        self
    }

    pub fn horizontal_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl<Renderer> Widget<WorkpadMessage, Renderer> for ActiveCell
where
    Renderer: iced::advanced::Renderer,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let state = if self.focused {
            State::focused()
        } else {
            State::new()
        };
        tree::State::new(state)
    }

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
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &<Renderer as iced::advanced::Renderer>::Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let editor = self.editor.borrow();
        let value = editor.value();
        let text = editor.contents();
        // TODO Allow font & size config
        let font = renderer.default_font();
        let size = self.font_size;

        let (cursor, offset) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            let editor = self.editor.borrow();
            match editor.cursor_state() {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) = measure_cursor_and_scroll_offset(
                        renderer, bounds, value, size, position, font,
                    );

                    let is_cursor_visible = ((focus.now - focus.updated_at).as_millis()
                        / CURSOR_BLINK_INTERVAL_MILLIS)
                        % 2
                        == 0;
                    let cursor = if is_cursor_visible {
                        Some((
                            Quad {
                                bounds: Rectangle {
                                    x: bounds.x + text_value_width,
                                    y: bounds.y,
                                    width: 1.0,
                                    height: bounds.height,
                                },
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            Color::BLACK, // TODO theme.value_color(style),
                        ))
                    } else {
                        None
                    };

                    (cursor, offset)
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(renderer, bounds, value, size, left, font);

                    let (right_position, right_offset) = measure_cursor_and_scroll_offset(
                        renderer, bounds, value, size, right, font,
                    );

                    let width = right_position - left_position;

                    (
                        Some((
                            Quad {
                                bounds: Rectangle {
                                    x: bounds.x + left_position,
                                    y: bounds.y,
                                    width,
                                    height: bounds.height,
                                },
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            Color::from_rgb8(20, 50, 20), // TODO theme.selection_color(style),
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                    )
                }
            }
        } else {
            (None, 0.0)
        };

        let text_width = renderer.measure_width(&text, size, font, text::Shaping::Advanced);

        let render = |renderer: &mut Renderer| {
            if let Some((cursor, color)) = cursor {
                renderer.fill_quad(cursor, color);
            } else {
                renderer.with_translation(Vector::ZERO, |_| {});
            }

            // TODO Colors
            // color: if text.is_empty() {
            //     theme.placeholder_color(style)
            // } else if is_disabled {
            //     theme.disabled_color(style)
            // } else {
            //     theme.value_color(style)
            // },
            let color = Color::BLACK;
            let (h_align, v_align, width) = if state.is_focused() {
                (
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                    f32::INFINITY,
                )
            } else {
                (
                    self.horizontal_alignment,
                    self.vertical_alignment,
                    bounds.width,
                )
            };
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
                renderer.with_translation(Vector::new(-offset, 0.0), render)
            });
        } else {
            render(renderer);
        }
    }

    fn diff(&self, _tree: &mut Tree) {}

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
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, WorkpadMessage>,
        _viewport: &Rectangle,
    ) -> Status {
        let mut publish = |msg: Option<WorkpadMessage>| {
            if let Some(m) = msg {
                shell.publish(m);
            }
        };

        let state = tree.state.downcast_mut::<State>();
        let focused = state.is_focused();

        match event {
            Event::Window(window::Event::Unfocused) => {
                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = false;
                }
                Status::Ignored
            }
            Event::Window(window::Event::Focused) => {
                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
                Status::Ignored
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(focus) = &mut state.is_focused {
                    if focus.is_window_focused {
                        focus.now = now;

                        let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                            - (now - focus.updated_at).as_millis() % CURSOR_BLINK_INTERVAL_MILLIS;

                        shell.request_redraw(window::RedrawRequest::At(
                            now + Duration::from_millis(millis_until_redraw as u64),
                        ));
                    }
                }
                Status::Ignored
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = modifiers;
                Status::Ignored
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if focused && !c.is_control() =>
            {
                let mut editor = self.editor.borrow_mut();
                editor.insert(c);
                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) if focused => {
                let modifiers = state.keyboard_modifiers;

                let mut editor = self.editor.borrow_mut();
                let jump = platform::is_jump_modifier_pressed(modifiers);
                match key_code {
                    keyboard::KeyCode::F2 => editor.toggle_edit_mode(),
                    keyboard::KeyCode::Escape if state.is_focused() => editor.abandon_editing(),
                    keyboard::KeyCode::Backspace if jump => editor.jump_backspace(),
                    keyboard::KeyCode::Backspace => editor.backspace(),
                    keyboard::KeyCode::Left => publish(editor.left()),
                    keyboard::KeyCode::Right => publish(editor.right()),
                    keyboard::KeyCode::Up => publish(editor.up()),
                    keyboard::KeyCode::Down => publish(editor.down()),
                    keyboard::KeyCode::Enter => publish(editor.enter()),
                    _ => {}
                };

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code: _key_code,
                ..
            }) if focused => {
                // match key_code {
                //     // keyboard::KeyCode::V => {
                //     //     state.is_pasting = None;
                //     // }
                //     // keyboard::KeyCode::Tab
                //     // | keyboard::KeyCode::Up
                //     // | keyboard::KeyCode::Down => {
                //     //     return event::Status::Ignored;
                //     // }
                //     _ => {}
                // }

                // //} else {
                // //state.is_pasting = None;
                Status::Captured
            }
            _ => Status::Ignored,
        }
    }
}

fn measure_cursor_and_scroll_offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    value: &Value,
    size: f32,
    cursor_index: usize,
    font: Renderer::Font,
) -> (f32, f32)
where
    Renderer: text::Renderer,
{
    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_width(&text_before_cursor, size, font, text::Shaping::Advanced);

    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

impl<Renderer> From<ActiveCell> for Element<'_, WorkpadMessage, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer: text::Renderer,
{
    fn from(grid: ActiveCell) -> Self {
        Self::new(grid)
    }
}

/// The state of a [`ActiveCell`].
#[derive(Debug, Clone)]
pub struct State {
    // value: Value,
    is_focused: Option<Focus>,
    // is_dragging: bool,
    #[allow(dead_code)] // TODO pasting
    is_pasting: Option<Value>,
    // last_click: Option<mouse::Click>,
    // cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // // TODO: Add stateful horizontal scrolling offset
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self {
            is_focused: None,
            // is_dragging: false,
            is_pasting: None,
            // last_click: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }

    pub fn focused() -> Self {
        let mut result = Self::new();
        result.focus();
        result
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Focuses the [`ActiveCell`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
            is_window_focused: true,
        });
    }

    // /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    // pub fn move_cursor_to_front(&mut self) {
    //     self.cursor.move_to(0);
    // }

    // /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    // pub fn move_cursor_to_end(&mut self) {
    //     self.cursor.move_to(usize::MAX);
    // }

    // /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    // pub fn move_cursor_to(&mut self, position: usize) {
    //     self.cursor.move_to(position);
    // }

    // /// Selects all the content of the [`TextInput`].
    // pub fn select_all(&mut self) {
    //     self.cursor.select_range(0, usize::MAX);
    // }
}
