use std::{rc::Rc, time::Instant};

use iced::{
    advanced::{
        layout::Node,
        mouse, overlay, renderer,
        text::{self, Paragraph},
        widget::{tree, Tree},
        Clipboard, Layout, Shell, Text, Widget,
    },
    event, touch,
    widget::text::LineHeight,
    Background, Color, Element, Event, Length, Point, Rectangle, Size, Vector,
};

use super::{
    MenuEntry, MenuOverlay, MenuStates, Roots, StyleSheet, CTRL_F2, DOWN, ENTER, ESCAPE, LEFT,
    MENUBAR_ENTRY_PADDING, MENUBAR_PADDING, MENU_PADDING, RIGHT, TEXT_SIZE_MENU,
};

pub(super) struct MenuBar<Message, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    font: Option<Renderer::Font>,
    roots: Rc<Roots<Message>>,
}

impl<Message, Renderer> MenuBar<Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub(super) fn new(roots: Rc<Roots<Message>>) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Shrink,
            font: None,
            roots,
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for MenuBar<Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuBarState::new())
    }

    fn diff(&self, tree: &mut Tree) {
        // Reset state if the tree is being rebuilt
        let state = tree.state.downcast_mut::<MenuBarState>();
        state.reset();
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let menubar_limits = limits
            .width(self.width)
            .height(self.height)
            .pad(MENUBAR_PADDING);

        let mut chidren = vec![];
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        let mut x = MENUBAR_PADDING.left;

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let entry_limits = menubar_limits
            .width(Length::Shrink)
            .height(Length::Shrink)
            .pad(MENUBAR_ENTRY_PADDING);

        for menu in self.roots.iter() {
            let paragraph = Renderer::Paragraph::with_text(Text {
                content: &menu.name,
                bounds: Size::INFINITY,
                size: TEXT_SIZE_MENU,
                line_height: LineHeight::Absolute(TEXT_SIZE_MENU),
                font,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
            });
            let text_size = paragraph.min_bounds();

            let entry_size = entry_limits.resolve(text_size).pad(MENUBAR_ENTRY_PADDING);
            let entry_position = Point::new(x, MENUBAR_PADDING.top);
            x += entry_size.width;
            width += entry_size.width;
            height = height.max(entry_size.height);

            let mut entry_layout = Node::new(entry_size);
            entry_layout.move_to(entry_position);
            chidren.push(entry_layout);
        }
        let content_size = Size::new(width, height);

        let menubar_size = menubar_limits.resolve(content_size).pad(MENUBAR_PADDING);
        Node::with_children(menubar_size, chidren)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced::advanced::Renderer>::Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let style: <Renderer::Theme as StyleSheet>::Style = Default::default();
        let menu_bar_appearance = theme.menu_bar(&style);
        let state = tree.state.downcast_ref::<MenuBarState>();

        // Background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            menu_bar_appearance.background,
        );

        for (index, (menu, layout)) in self.roots.iter().zip(layout.children()).enumerate() {
            let bounds = layout.bounds();

            let item_appearance = match state.active.is_open_at(index) {
                true => theme.selected_menu_bar_item(&style),
                false => theme.active_menu_bar_item(&style),
            };

            // Background
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: item_appearance.border_radius.into(),
                    border_width: item_appearance.border_width,
                    border_color: item_appearance.border_color,
                },
                item_appearance.background,
            );

            let text_size = Size::new(
                bounds.size().width - MENUBAR_ENTRY_PADDING.horizontal(),
                bounds.size().height - MENUBAR_ENTRY_PADDING.vertical(),
            );

            let text_bounds = Rectangle::new(
                Point::new(
                    bounds.x + MENUBAR_ENTRY_PADDING.left,
                    bounds.y + MENUBAR_ENTRY_PADDING.top,
                ),
                text_size,
            );

            let text_position = Point::new(
                text_bounds.position().x + text_size.width / 2.0,
                text_bounds.position().y + text_size.height,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: text_bounds,
                    border_radius: item_appearance.border_radius.into(),
                    border_width: item_appearance.border_width,
                    border_color: item_appearance.border_color,
                },
                item_appearance.background,
            );

            let font = self.font.unwrap_or_else(|| renderer.default_font());
            renderer.fill_text(
                Text {
                    content: &menu.name,
                    font,
                    bounds: text_bounds.size(),
                    size: TEXT_SIZE_MENU,
                    line_height: LineHeight::Absolute(TEXT_SIZE_MENU),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    shaping: text::Shaping::Advanced,
                },
                text_position,
                item_appearance.text_color,
                text_bounds,
            );
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let entries_len = self.roots.len();
        if entries_len == 0 {
            return event::Status::Ignored;
        }

        let state = tree.state.downcast_mut::<MenuBarState>();

        match event {
            Event::Keyboard(key_event) => match key_event {
                // The menu bar is triggered by a double CTRL-F2.  Since CTRL-F2 invokes the real menu bar we get:
                //
                // CTRL-F2 pressed  - real menu bar get focus and we don't see the event
                // CTRL-F2 released - real menu bar has focus and we don't see the event
                // CTRL-F2 pressed  - real menu bar loses focus consuming the event so again we don't see it
                // CTRL-F2 released - focus has been returned to the app and we see this event an duse it to toggle focus
                CTRL_F2 => {
                    if state.active.is_open() {
                        state.active = Activation::Closed;
                        state.menu_states.clear_after(0);
                    } else {
                        state.active = Activation::Open(0);
                    }
                    event::Status::Captured
                }
                ESCAPE => {
                    state.active = Activation::Closed;
                    event::Status::Captured
                }
                ENTER | DOWN if state.menu_states.depth() == 0 && state.active.is_open() => {
                    let active = state.active.open_at();
                    state.menu_states.push(self.roots[active].profile());
                    state.menu_states.focus_next(true);
                    event::Status::Captured
                }
                DOWN if state.menu_states.depth() > 0 && state.active.is_open() => {
                    state.menu_states.focus_next(true);
                    event::Status::Captured
                }
                RIGHT if state.active.is_open() => {
                    let active = (state.active.open_at() + 1) % entries_len;
                    state.active = Activation::Open(active);
                    if state.menu_states.depth() > 0 {
                        state.menu_states.clear_after(0);
                        state.menu_states.push(self.roots[active].profile());
                    }
                    event::Status::Captured
                }
                LEFT if state.active.is_open() => {
                    let active = (state.active.open_at() + entries_len - 1) % entries_len;
                    state.active = Activation::Open(active);
                    if state.menu_states.depth() > 0 {
                        state.menu_states.clear_after(0);
                        state.menu_states.push(self.roots[active].profile());
                    }
                    event::Status::Captured
                }
                _ => event::Status::Ignored,
            },
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    let over = layout.children().position(|n| cursor.is_over(n.bounds()));
                    match over {
                        Some(over) => match state.active {
                            Activation::Open(active) if active != over => {
                                state.active = Activation::Open(over);
                                state.menu_states.clear_after(0);
                                state.menu_states.push(self.roots[over].profile());
                                state.menu_states.focus_next(false);
                            }
                            Activation::Closed | Activation::Open(_) => {}
                            Activation::HiddenByMouseMove => {
                                state.active = Activation::Open(over);
                                state.menu_states.push(self.roots[over].profile());
                                state.menu_states.focus_next(false);
                            }
                        },
                        None => match state.active {
                            Activation::Open(_) => {
                                state.active = Activation::HiddenByMouseMove;
                                state.menu_states.clear_after(0);
                            }
                            Activation::HiddenByMouseMove | Activation::Closed => {}
                        },
                    };
                    return event::Status::Captured;
                }
                event::Status::Ignored
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) && !state.active.is_open() {
                    state.pressed = Some((cursor.position().unwrap(), Instant::now()));
                    if let Some(over) = layout.children().position(|n| cursor.is_over(n.bounds())) {
                        if !state.active.is_open_at(over) {
                            state.active = Activation::Open(over);
                            state.menu_states.clear_after(0);
                            state.menu_states.push(self.roots[over].profile());
                            state.menu_states.focus_next(false);
                        }
                    }
                    event::Status::Captured
                } else {
                    state.pressed = None;
                    state.active = Activation::Closed;
                    state.menu_states.clear_after(0);
                    event::Status::Ignored
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) && state.active.is_open() {
                    let valid_click = if let Some((position, at)) = state.pressed {
                        position == cursor.position().unwrap()
                            && Instant::now().duration_since(at).as_millis() < 300
                    } else {
                        false
                    };
                    if !valid_click {
                        state.active = Activation::Closed;
                        state.menu_states.clear_after(0);
                    }
                    return event::Status::Captured;
                }
                state.pressed = None;
                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_ref::<MenuBarState>();
        if state.menu_states.depth() == 0 {
            return None;
        }

        let mut elements = vec![];

        let mut menu = state.active.map(|i| &self.roots[i]);
        let mut position = state
            .active
            .map(|i| {
                layout
                    .children()
                    .nth(i)
                    .expect("Expected menubar entry layout")
            })
            .map(|l| l.bounds())
            .map(|b| b.position() + Vector::new(0.0, b.size().height))
            .unwrap_or(Point::ORIGIN);
        let mut depth = 0;

        while let Some(mnu) = menu {
            depth += 1;
            if depth > state.menu_states.depth() {
                break;
            }

            let font = self.font.unwrap_or_else(|| renderer.default_font());

            let entries = mnu.entries();
            menu = match state.menu_states.selected(depth) {
                Some(selected) => {
                    if let MenuEntry::SubMenu(ref menu) = entries[selected] {
                        Some(menu)
                    } else {
                        None
                    }
                }
                None => None,
            };
            let overlay = MenuOverlay::new(entries, font, Rc::clone(&state.menu_states), depth);
            let mut element = overlay::Element::new(position, Box::new(overlay));
            let layout = element.layout(renderer, Size::INFINITY, Vector::ZERO);
            if let Some(index) = state.menu_states.selected(depth) {
                let child_layout = &layout.children()[index];
                position = position
                    + Vector::new(child_layout.size().width, child_layout.bounds().y)
                    + Vector::new(MENU_PADDING.right, -MENU_PADDING.top);
            }

            elements.push(element);
        }

        match elements.len() {
            0 => None,
            1 => Some(elements.into_iter().next().unwrap()),
            _ => Some(overlay::Group::with_children(elements).into()),
        }
    }
}

impl<'a, Message, Renderer> From<MenuBar<Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(menu: MenuBar<Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(menu)
    }
}

// Models the manner in which the menu bar items are activated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Activation {
    Open(usize),
    HiddenByMouseMove,
    #[default]
    Closed,
}

impl Activation {
    fn is_open(&self) -> bool {
        matches!(self, Activation::Open(_))
    }

    fn is_open_at(&self, index: usize) -> bool {
        match *self {
            Activation::Open(open) => open == index,
            Activation::HiddenByMouseMove | Activation::Closed => false,
        }
    }

    fn open_at(&self) -> usize {
        match *self {
            Activation::Open(open) => open,
            Activation::HiddenByMouseMove | Activation::Closed => panic!("Activation is not open"),
        }
    }

    fn map<T>(&self, f: impl Fn(usize) -> T) -> Option<T> {
        match *self {
            Activation::Open(open) => Some(f(open)),
            Activation::HiddenByMouseMove | Activation::Closed => None,
        }
    }
}

#[derive(Debug)]
struct MenuBarState {
    active: Activation,
    menu_states: Rc<MenuStates>,
    pressed: Option<(Point, Instant)>,
}

impl MenuBarState {
    fn new() -> Self {
        Self {
            active: Default::default(),
            menu_states: Rc::new(MenuStates::new()),
            pressed: None,
        }
    }

    fn reset(&mut self) {
        self.active = Default::default();
        self.menu_states.clear_after(0);
        self.pressed = None;
    }
}

/// The appearance of the [`MenuBar`] and [`Menus`].
#[derive(Clone, Copy, Debug)]
pub struct MenuBarAppearance {
    /// The background of the [`MenuBar`]
    pub background: Background,
}

/// The appearance of an item in a [`MenuBar`].
#[derive(Clone, Copy, Debug)]
pub struct MenuBarItemAppearance {
    /// The background of an item in a [`MenuBar`]
    pub background: Background,

    /// The text color of an item in a [`MenuBar`]
    pub text_color: Color,

    /// The border radius of an item on a [`MenuBar`]
    pub border_radius: f32,

    /// The border width of an item on a [`MenuBar`]
    pub border_width: f32,

    /// The border color of an item on a [`MenuBar`]
    pub border_color: Color,
}
