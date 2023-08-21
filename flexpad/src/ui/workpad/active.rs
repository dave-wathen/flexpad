use std::time::{Duration, Instant};

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
use cursor::Cursor;
use editor::Editor;

pub struct ActiveCell<'a, Message> {
    value: Value,
    // is_secure: bool,
    // font: Option<Renderer::Font>,
    // width: Length,
    // padding: Padding,
    // size: Option<f32>,
    // line_height: text::LineHeight,
    on_new_value: Option<Box<dyn Fn(String) -> Message + 'a>>,
    // on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    // on_submit: Option<Message>,
    // icon: Option<Icon<Renderer::Font>>,
    // style: <Renderer::Theme as StyleSheet>::Style,
    on_move: Option<Box<dyn Fn(Move) -> Message + 'a>>,
}

impl<'a, Message> ActiveCell<'a, Message> {
    pub fn new(value: &str) -> Self {
        Self {
            value: Value::new(value),
            on_new_value: None,
            on_move: None,
        }
    }

    pub fn on_move(mut self, f: impl Fn(Move) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(f));
        self
    }

    pub fn on_new_value(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_new_value = Some(Box::new(f));
        self
    }
}

impl<'a, Message: 'a, Renderer> Widget<Message, Renderer> for ActiveCell<'a, Message>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.value.clone()))
    }

    // fn diff(&self, tree: &mut Tree) {
    //     let state = tree.state.downcast_mut::<State>();

    //     // Unfocus text input if it becomes disabled
    //     if self.on_input.is_none() {
    //         state.last_click = None;
    //         state.is_focused = None;
    //         state.is_pasting = None;
    //         state.is_dragging = false;
    //     }
    // }

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

        let value = &state.value;
        let text = value.to_string();
        // TODO Allow font & size config
        let font = renderer.default_font();
        let size = 10.0; //renderer.default_size();

        let (cursor, offset) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            match state.cursor.state(value) {
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

            renderer.fill_text(Text {
                content: &text,
                // TODO Colors
                // color: if text.is_empty() {
                //     theme.placeholder_color(style)
                // } else if is_disabled {
                //     theme.disabled_color(style)
                // } else {
                //     theme.value_color(style)
                // },
                color: Color::BLACK,
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

    // fn operate(
    //     &self,
    //     _state: &mut Tree,
    //     _layout: Layout<'_>,
    //     _renderer: &Renderer,
    //     _operation: &mut dyn iced::advanced::widget::Operation<Message>,
    // ) {
    // }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
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
            Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
                let state = tree.state.downcast_mut::<State>();

                if !state.is_focused() {
                    state.focus();
                    state.value = Value::new("");
                    state.cursor = Cursor::default();
                };

                if let Some(focus) = &mut state.is_focused {
                    if state.is_pasting.is_none()
                        && !state.keyboard_modifiers.command()
                        && !c.is_control()
                    {
                        let mut editor = Editor::new(&mut state.value, &mut state.cursor);

                        editor.insert(c);
                        state.value = Value::new(&editor.contents());
                        focus.updated_at = Instant::now();
                    }
                }
                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                let state = tree.state.downcast_mut::<State>();
                let modifiers = state.keyboard_modifiers;

                if key_code == keyboard::KeyCode::F2 {
                    if state.is_focused() {
                        if let Some(ref mut focus) = state.is_focused {
                            if focus.mode == Mode::InternalNavigation {
                                focus.mode = Mode::TerminalNavigation
                            } else {
                                focus.mode = Mode::InternalNavigation
                            }
                        }
                    } else {
                        state.focus();
                        state.value = self.value.clone();
                        state.cursor = Cursor::default();
                        state.cursor.move_to(state.value.len());
                        if let Some(ref mut focus) = state.is_focused {
                            focus.mode = Mode::InternalNavigation;
                        }
                    }
                } else if key_code == keyboard::KeyCode::Backspace {
                    let value = &mut state.value;
                    if platform::is_jump_modifier_pressed(modifiers)
                        && state.cursor.selection(value).is_none()
                    {
                        state.cursor.select_left_by_words(value);
                    }

                    let mut editor = Editor::new(value, &mut state.cursor);
                    editor.backspace();
                } else {
                    // TODO Depends on mode
                    let mve = match key_code {
                        keyboard::KeyCode::Left => Some(Move::Left),
                        keyboard::KeyCode::Right => Some(Move::Right),
                        keyboard::KeyCode::Up => Some(Move::Up),
                        keyboard::KeyCode::Down => Some(Move::Down),
                        _ => None,
                    };

                    if let Some(mve) = mve {
                        if state.is_focused() {
                            notify_on_new_value(&self.on_new_value, state.unfocus(), shell)
                        }
                        notify_on_move(&self.on_move, mve, shell);
                    }
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

    // fn mouse_interaction(
    //     &self,
    //     _state: &Tree,
    //     _layout: Layout<'_>,
    //     _cursor: iced::advanced::mouse::Cursor,
    //     _viewport: &Rectangle,
    //     _renderer: &Renderer,
    // ) -> iced::advanced::mouse::Interaction {
    //     iced::advanced::mouse::Interaction::Idle
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

fn notify_on_move<Message>(
    on_move: &Option<Box<dyn Fn(Move) -> Message + '_>>,
    mve: Move,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_move) = on_move {
        shell.publish(on_move(mve));
    }
}

fn notify_on_new_value<Message>(
    on_new_value: &Option<Box<dyn Fn(String) -> Message + '_>>,
    new_value: String,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_new_value) = on_new_value {
        shell.publish(on_new_value(new_value));
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

impl<'a, Message, Renderer> From<ActiveCell<'a, Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer: text::Renderer,
{
    fn from(grid: ActiveCell<'a, Message>) -> Self {
        Self::new(grid)
    }
}

/// The state of a [`ActiveCell`].
#[derive(Debug, Clone)]
pub struct State {
    value: Value,
    is_focused: Option<Focus>,
    // is_dragging: bool,
    is_pasting: Option<Value>,
    // last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // // TODO: Add stateful horizontal scrolling offset
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    InternalNavigation,
    TerminalNavigation,
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
    mode: Mode,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new(value: Value) -> Self {
        Self {
            value,
            is_focused: None,
            // is_dragging: false,
            is_pasting: None,
            // last_click: None,
            cursor: Cursor::default(),
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
            mode: Mode::TerminalNavigation,
        });

        //        self.move_cursor_to_end();
    }

    /// Unfocuses the [`ActiveCell`].
    pub fn unfocus(&mut self) -> String {
        self.is_focused = None;
        self.value.to_string()
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

mod platform {
    use iced::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}
