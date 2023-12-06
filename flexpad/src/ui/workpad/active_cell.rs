use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse::{self, click},
        renderer::{Quad, Style},
        text,
        widget::{self, tree, Operation, Tree},
        Clipboard, Layout, Shell, Text, Widget,
    },
    alignment,
    event::Status,
    keyboard, touch,
    widget::{text::LineHeight, text_input::Value},
    window, Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector,
};

mod cursor;
mod editor;
mod platform;
pub use editor::Editor;

use super::active_sheet::Message;

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

pub struct ActiveCell<Renderer>
where
    Renderer: text::Renderer,
{
    id: Option<Id>,
    focused: bool,
    editor: Rc<RefCell<Editor>>,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    font_size: f32,
    edit_when_clicked: click::Kind,
}

impl<Renderer> ActiveCell<Renderer>
where
    Renderer: text::Renderer,
{
    /// Creates a new [`ActiveCell`].
    pub fn new(editor: Rc<RefCell<Editor>>) -> Self {
        Self {
            id: None,
            focused: false,
            editor,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            font: None,
            font_size: 10.0,
            edit_when_clicked: click::Kind::Single,
        }
    }

    /// Sets the [`Id`] of the [`ActiveCell`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the focus state of the [`ActiveCell`].
    pub fn focused(mut self, is_focused: bool) -> Self {
        self.focused = is_focused;
        self
    }

    pub fn edit_when_clicked(mut self, kind: click::Kind) -> Self {
        self.edit_when_clicked = kind;
        self
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

impl<Renderer> Widget<Message, Renderer> for ActiveCell<Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn width(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn height(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn state(&self) -> tree::State {
        let state = if self.focused {
            State::focused()
        } else {
            State::new()
        };
        tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Ensure state reflects widget focus
        if self.focused {
            state.focus();
        } else {
            state.unfocus();
        }
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
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let size = self.font_size;

        let (cursor, offset) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            let editor = self.editor.borrow();
            match editor.cursor_state() {
                None => (None, 0.0),
                Some(cursor::State::Index(position)) => {
                    let (text_value_width, offset) = measure_cursor_and_scroll_offset(
                        renderer, bounds, value, size, position, font,
                    );

                    let is_cursor_visible = editor.is_editing()
                        && ((focus.now - focus.updated_at).as_millis()
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
                Some(cursor::State::Selection { start, end }) => {
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
            let (h_align, v_align, width) = if state.is_focused() && editor.is_editing() {
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
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let mut publish = |msg: Option<Message>| {
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let click_position = cursor.position_over(layout.bounds());
                if let Some(cursor_position) = click_position {
                    if !focused {
                        if let Some(ref id) = self.id {
                            shell.publish(Message::Focus(id.0.clone()))
                        }
                    }

                    let target = cursor_position.x - layout.bounds().x;
                    let click = mouse::Click::new(cursor_position, state.last_click);

                    let font = self.font.unwrap_or_else(|| renderer.default_font());
                    let size = self.font_size;
                    if self.editor.borrow().is_editing() {
                        match click.kind() {
                            click::Kind::Single => {
                                let position = if target > 0.0 {
                                    find_cursor_position(
                                        renderer,
                                        layout.bounds(),
                                        font,
                                        size,
                                        LineHeight::default(),
                                        &self.editor.borrow(),
                                        target,
                                    )
                                } else {
                                    None
                                }
                                .unwrap_or(0);

                                let mut editor = self.editor.borrow_mut();
                                if editor.is_editing() {
                                    if state.keyboard_modifiers.shift() {
                                        editor.select_to(position);
                                    } else {
                                        editor.move_to(position);
                                    }
                                    state.is_dragging = true;
                                } else {
                                    editor.start_edit();
                                }
                            }
                            click::Kind::Double => {
                                let position = find_cursor_position(
                                    renderer,
                                    layout.bounds(),
                                    font,
                                    size,
                                    LineHeight::default(),
                                    &self.editor.borrow(),
                                    target,
                                )
                                .unwrap_or(0);

                                let mut editor = self.editor.borrow_mut();
                                editor.select_word_at(position);
                                state.is_dragging = false;
                            }
                            click::Kind::Triple => {
                                let mut editor = self.editor.borrow_mut();
                                editor.select_all();
                                state.is_dragging = false;
                            }
                        }
                    } else {
                        // Since Eq not implemented for click::Kind at the time of writing
                        let start_edit = matches!(
                            (click.kind(), self.edit_when_clicked),
                            (click::Kind::Single, click::Kind::Single)
                                | (click::Kind::Double, click::Kind::Double)
                                | (click::Kind::Triple, click::Kind::Triple)
                        );
                        if start_edit {
                            let mut editor = self.editor.borrow_mut();
                            editor.start_edit();
                        }
                    }

                    state.last_click = Some(click);
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = modifiers;
                Status::Ignored
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if focused
                    && !c.is_control()
                    && !state.keyboard_modifiers.alt()
                    && !state.keyboard_modifiers.command()
                    && !state.keyboard_modifiers.control() =>
            {
                let mut editor = self.editor.borrow_mut();
                editor.insert(c);
                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) if focused => {
                state.keyboard_modifiers = modifiers;

                let mut editor = self.editor.borrow_mut();
                let jump = platform::is_jump_modifier_pressed(modifiers);
                let shift = modifiers.shift();
                match key_code {
                    // Editing
                    keyboard::KeyCode::F2 => editor.start_edit(),
                    keyboard::KeyCode::Escape if state.is_focused() => editor.abandon_editing(),
                    keyboard::KeyCode::Backspace if jump => editor.jump_backspace(),
                    keyboard::KeyCode::Backspace => editor.backspace(),
                    keyboard::KeyCode::Enter | keyboard::KeyCode::NumpadEnter if shift => {
                        publish(editor.back_enter())
                    }
                    keyboard::KeyCode::Tab if shift => publish(editor.back_tab()),
                    keyboard::KeyCode::Enter | keyboard::KeyCode::NumpadEnter => {
                        publish(editor.enter())
                    }
                    keyboard::KeyCode::Tab => publish(editor.tab()),
                    // Selection
                    keyboard::KeyCode::Left if jump && shift => editor.select_left_by_words(),
                    keyboard::KeyCode::Right if jump && shift => editor.select_right_by_words(),
                    keyboard::KeyCode::Left if shift => editor.select_left(),
                    keyboard::KeyCode::Right if shift => editor.select_right(),
                    // Navigation
                    keyboard::KeyCode::Left if jump => publish(editor.jump_left()),
                    keyboard::KeyCode::Right if jump => publish(editor.jump_right()),
                    keyboard::KeyCode::Up if jump => publish(editor.jump_up()),
                    keyboard::KeyCode::Down if jump => publish(editor.jump_down()),
                    keyboard::KeyCode::Left => publish(editor.left()),
                    keyboard::KeyCode::Right => publish(editor.right()),
                    keyboard::KeyCode::Up => publish(editor.up()),
                    keyboard::KeyCode::Down => publish(editor.down()),
                    _ => {}
                };

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code: _key_code,
                modifiers,
            }) if focused => {
                state.keyboard_modifiers = modifiers;
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

// TODO Convert this to work for ActiveCell and then use in Single click to position in text
/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
fn find_cursor_position<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: f32,
    line_height: text::LineHeight,
    editor: &Editor,
    x: f32,
) -> Option<usize>
where
    Renderer: text::Renderer,
{
    let offset = offset(renderer, text_bounds, font, size, editor);
    let value = editor.contents();

    let char_offset = renderer
        .hit_test(
            &value,
            size,
            line_height,
            font,
            Size::INFINITY,
            text::Shaping::Advanced,
            Point::new(x + offset, text_bounds.height / 2.0),
            true,
        )
        .map(text::Hit::cursor)?;

    Some(unicode_segmentation::UnicodeSegmentation::graphemes(&value[..char_offset], true).count())
}

fn offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: f32,
    editor: &Editor,
) -> f32
where
    Renderer: text::Renderer,
{
    let cursor = editor.cursor();
    let value = editor.value();

    let focus_position = match cursor.state(value) {
        cursor::State::Index(i) => i,
        cursor::State::Selection { end, .. } => end,
    };

    let (_, offset) =
        measure_cursor_and_scroll_offset(renderer, text_bounds, value, size, focus_position, font);

    offset
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

impl<'a, Renderer> From<ActiveCell<Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Renderer: text::Renderer,
{
    fn from(cell: ActiveCell<Renderer>) -> Self {
        Self::new(cell)
    }
}

/// The state of a [`ActiveCell`].
#[derive(Debug, Clone)]
pub struct State {
    is_focused: Option<Focus>,
    is_dragging: bool,
    #[allow(dead_code)] // TODO pasting
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    keyboard_modifiers: keyboard::Modifiers,
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`ActiveCell`].
    pub fn new() -> Self {
        Self {
            is_focused: None,
            is_dragging: false,
            is_pasting: None,
            last_click: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }

    /// Creates a new [`State`], representing an focused [`ActiveCell`].
    pub fn focused() -> Self {
        let mut result = Self::new();
        result.focus();
        result
    }

    /// Returns whether the [`ActiveCell`] is currently focused or not.
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
