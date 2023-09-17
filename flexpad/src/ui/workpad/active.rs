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
        widget::{tree, Tree},
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

pub struct ActiveCell {
    editor: Rc<RefCell<Editor>>,
}

impl ActiveCell {
    pub fn new(editor: Rc<RefCell<Editor>>) -> Self {
        Self { editor }
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
        tree::State::new(State::new())
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
        let size = 10.0; //renderer.default_size();

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
            let text = if state.is_focused() {
                Text {
                    content: &text,
                    color,
                    font,
                    bounds: Rectangle {
                        y: bounds.center_y(),
                        width: f32::INFINITY,
                        ..bounds
                    },
                    size,
                    line_height: LineHeight::default(),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                }
            } else {
                Text {
                    content: &text,
                    color,
                    font,
                    bounds: Rectangle {
                        y: bounds.center_y(),
                        x: bounds.center_x(),
                        ..bounds
                    },
                    size,
                    line_height: LineHeight::default(),
                    // TODO Set from data when not focused
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                }
            };
            renderer.fill_text(text);
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

        match event {
            Event::Window(window::Event::Unfocused) => {
                let state = tree.state.downcast_mut::<State>();

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = false;
                }
                Status::Ignored
            }
            Event::Window(window::Event::Focused) => {
                let state = tree.state.downcast_mut::<State>();

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
                Status::Ignored
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = tree.state.downcast_mut::<State>();

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
                let state = tree.state.downcast_mut::<State>();
                state.keyboard_modifiers = modifiers;
                Status::Ignored
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c)) if !c.is_control() => {
                let state = tree.state.downcast_mut::<State>();
                let mut editor = self.editor.borrow_mut();
                editor.insert(c);
                if !state.is_focused() && editor.is_editing() {
                    state.focus();
                }
                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                let state = tree.state.downcast_mut::<State>();
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

                if !state.is_focused() && editor.is_editing() {
                    state.focus();
                } else if state.is_focused() && !editor.is_editing() {
                    state.unfocus();
                }

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code: _key_code,
                ..
            }) => {
                let state = tree.state.downcast_mut::<State>();

                if state.is_focused.is_some() {
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
                } else {
                    Status::Ignored
                }
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

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    // /// Returns the [`Cursor`] of the [`TextInput`].
    // pub fn cursor(&self) -> Cursor {
    //     self.cursor
    // }

    /// Focuses the [`ActiveCell`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
            is_window_focused: true,
        });

        //        self.move_cursor_to_end();
    }

    /// Unfocuses the [`ActiveCell`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
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
