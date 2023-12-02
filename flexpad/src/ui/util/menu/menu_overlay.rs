use std::rc::Rc;

use iced::{
    advanced::{
        self,
        layout::{self, Limits, Node},
        mouse, overlay, renderer, text, Clipboard, Layout, Shell, Text,
    },
    event, touch,
    widget::text::LineHeight,
    Background, Color, Event, Length, Pixels, Point, Rectangle, Size, Vector,
};

use crate::ui::util::SPACE_L;

use super::{
    MenuEntry, MenuStates, StyleSheet, DOWN, ENTER, LEFT, MENU_ENTRY_PADDING, MENU_PADDING, RIGHT,
    SEPARATOR_HEIGHT, TEXT_SIZE_MENU, UP,
};

pub(super) struct MenuOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer,
    Renderer: advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    entries: Vec<&'a MenuEntry<Message>>,
    font: Renderer::Font,
    menu_states: Rc<MenuStates>,
    depth: usize,
}

impl<'a, Message, Renderer> MenuOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer,
    Renderer: advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub(super) fn new(
        entries: Vec<&'a MenuEntry<Message>>,
        font: Renderer::Font,
        menu_states: Rc<MenuStates>,
        depth: usize,
    ) -> Self {
        Self {
            entries,
            font,
            menu_states,
            depth,
        }
    }

    fn move_up(&mut self) {
        self.menu_states.move_up();
    }

    fn move_down(&mut self) {
        self.menu_states.move_down();
    }

    fn entry_sizes(&self, renderer: &Renderer, limits: Limits) -> Vec<Size> {
        let sizes: Vec<(Size, Size)> = self
            .entries
            .iter()
            .map(|e| match e {
                MenuEntry::SectionStart(_) => (Size::new(0.0, SEPARATOR_HEIGHT), Size::ZERO),
                MenuEntry::SubMenu(menu) => {
                    let text_size = renderer.measure(
                        &menu.name,
                        TEXT_SIZE_MENU,
                        LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                        self.font,
                        limits.max(),
                        text::Shaping::Basic,
                    );
                    (text_size, Size::ZERO)
                }
                MenuEntry::Item(name, shortcut, _) => {
                    let text_size = renderer.measure(
                        name,
                        TEXT_SIZE_MENU,
                        LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                        self.font,
                        limits.max(),
                        text::Shaping::Basic,
                    );

                    let shorcut_size = if let Some(key) = shortcut {
                        renderer.measure(
                            &key.to_string(),
                            TEXT_SIZE_MENU,
                            LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                            self.font,
                            limits.max(),
                            text::Shaping::Basic,
                        )
                    } else {
                        Size::ZERO
                    };

                    (text_size, shorcut_size)
                }
            })
            .collect();

        let text_width = sizes.iter().fold(0.0_f32, |w, s| w.max(s.0.width));
        let shortcut_width = sizes.iter().fold(0.0_f32, |w, s| w.max(s.1.width));
        let width = text_width + SPACE_L + shortcut_width;

        sizes
            .into_iter()
            .map(|s| Size::new(width, s.0.height).pad(MENU_ENTRY_PADDING))
            .collect()
    }
}

impl<'a, Message, Renderer> overlay::Overlay<Message, Renderer>
    for MenuOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: advanced::Renderer,
    Renderer: advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> iced::advanced::layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .pad(MENU_PADDING);

        // TODO Store the layout in children and get rid of entry_sizes?
        let sizes = self.entry_sizes(renderer, limits);
        let mut children = vec![];
        let mut entries_size = Size::ZERO;
        let x = MENU_PADDING.left;
        let mut y = MENU_PADDING.top;
        for size in sizes {
            entries_size.height += size.height;
            entries_size.width = entries_size.width.max(size.width);
            let mut child = Node::new(size);
            child.move_to(Point::new(x, y));
            children.push(child);
            y += size.height;
        }

        Node::with_children(entries_size.pad(MENU_PADDING), children)
            .translate(Vector::new(position.x, position.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &<Renderer as renderer::Renderer>::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let style: <Renderer::Theme as StyleSheet>::Style = Default::default();
        let focused = self.depth == self.menu_states.focused();
        let menu_appearance = theme.menu(&style);

        // Containing box
        // TODO Shadows
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_radius: menu_appearance.border_radius.into(),
                border_width: menu_appearance.border_width,
                border_color: menu_appearance.border_color,
            },
            menu_appearance.background,
        );

        let selected = self.menu_states.selected(self.depth);
        for (index, (entry, child_layout)) in self.entries.iter().zip(layout.children()).enumerate()
        {
            let bounds = child_layout.bounds();
            let selected = selected == Some(index);
            let item_appearance = match (entry.is_active(), selected, focused) {
                (true, true, true) => theme.focused_selected_menu_item(&style),
                (true, true, false) => theme.unfocused_selected_menu_item(&style),
                (true, false, _) => theme.active_menu_item(&style),
                _ => theme.inactive_menu_item(&style),
            };

            // Item background
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: item_appearance.border_radius.into(),
                    border_width: item_appearance.border_width,
                    border_color: item_appearance.border_color,
                },
                item_appearance.background,
            );

            let inner_bounds = Rectangle::new(
                Point::new(
                    bounds.x + MENU_ENTRY_PADDING.left,
                    bounds.y + MENU_ENTRY_PADDING.top,
                ),
                Size::new(
                    bounds.width - MENU_ENTRY_PADDING.left - MENU_ENTRY_PADDING.right,
                    bounds.height - MENU_ENTRY_PADDING.top - MENU_ENTRY_PADDING.bottom,
                ),
            );

            let text_bounds = Rectangle::new(
                Point::new(inner_bounds.x, inner_bounds.y + inner_bounds.height),
                inner_bounds.size(),
            );

            match entry {
                MenuEntry::SectionStart(_) => {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: inner_bounds,
                            border_radius: 0.0.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                        menu_appearance.separator_fill,
                    );
                }
                MenuEntry::SubMenu(menu) => {
                    renderer.fill_text(Text {
                        content: &menu.name,
                        color: item_appearance.text_color,
                        font: self.font,
                        bounds: text_bounds,
                        size: TEXT_SIZE_MENU,
                        line_height: LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                        horizontal_alignment: iced::alignment::Horizontal::Left,
                        vertical_alignment: iced::alignment::Vertical::Bottom,
                        shaping: text::Shaping::Advanced,
                    });

                    let text_bounds = Rectangle::new(
                        Point::new(
                            inner_bounds.x + inner_bounds.width,
                            inner_bounds.y + inner_bounds.height,
                        ),
                        inner_bounds.size(),
                    );

                    renderer.fill_text(Text {
                        content: ">",
                        color: item_appearance.text_color,
                        font: self.font,
                        bounds: text_bounds,
                        size: TEXT_SIZE_MENU,
                        line_height: LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                        horizontal_alignment: iced::alignment::Horizontal::Right,
                        vertical_alignment: iced::alignment::Vertical::Bottom,
                        shaping: text::Shaping::Advanced,
                    });
                }
                MenuEntry::Item(name, shortcut, _) => {
                    renderer.fill_text(Text {
                        content: name,
                        color: item_appearance.text_color,
                        font: self.font,
                        bounds: text_bounds,
                        size: TEXT_SIZE_MENU,
                        line_height: LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                        horizontal_alignment: iced::alignment::Horizontal::Left,
                        vertical_alignment: iced::alignment::Vertical::Bottom,
                        shaping: text::Shaping::Advanced,
                    });

                    if let Some(key) = shortcut {
                        let text_bounds = Rectangle::new(
                            Point::new(
                                inner_bounds.x + inner_bounds.width,
                                inner_bounds.y + inner_bounds.height,
                            ),
                            inner_bounds.size(),
                        );

                        renderer.fill_text(Text {
                            content: &key.to_string(),
                            color: item_appearance.shortcut_color,
                            font: self.font,
                            bounds: text_bounds,
                            size: TEXT_SIZE_MENU,
                            line_height: LineHeight::Absolute(Pixels(TEXT_SIZE_MENU)),
                            horizontal_alignment: iced::alignment::Horizontal::Right,
                            vertical_alignment: iced::alignment::Vertical::Bottom,
                            shaping: text::Shaping::Advanced,
                        });
                    }
                }
            }
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let selected = self.menu_states.selected(self.depth);
        match event {
            Event::Keyboard(keyboard_event) => {
                if self.depth != self.menu_states.focused() {
                    return event::Status::Ignored;
                }

                match keyboard_event {
                    UP | DOWN => {
                        if keyboard_event == UP {
                            self.move_up()
                        } else {
                            self.move_down()
                        };
                        if let Some(selected) = self.menu_states.selected(self.depth) {
                            self.menu_states.clear_after(self.depth);
                            if let MenuEntry::SubMenu(submenu) = self.entries[selected] {
                                self.menu_states.push(submenu.profile());
                            }
                        }
                        event::Status::Captured
                    }
                    RIGHT if selected.is_some() => {
                        if let Some(selected) = self.menu_states.selected(self.depth) {
                            if let MenuEntry::SubMenu(submenu) = self.entries[selected] {
                                if self.depth == self.menu_states.depth() {
                                    self.menu_states.push(submenu.profile());
                                }
                                self.menu_states.focus_next(true);
                                return event::Status::Captured;
                            }
                        }
                        event::Status::Ignored
                    }
                    LEFT if self.depth > 1 => {
                        self.menu_states.focus_previous();
                        event::Status::Captured
                    }
                    ENTER => {
                        if let Some(selected) = self.menu_states.selected(self.depth) {
                            if let MenuEntry::Item(_, _, on_select) = self.entries[selected] {
                                if let Some(msg) = on_select.clone() {
                                    shell.publish(msg);
                                    return event::Status::Captured;
                                }
                            }
                        }
                        event::Status::Ignored
                    }
                    _ => event::Status::Ignored,
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    if self.menu_states.focused() != self.depth {
                        self.menu_states.focus_on(self.depth);
                    }

                    let over = layout.children().position(|n| cursor.is_over(n.bounds()));
                    if let Some(over) = over {
                        if self.entries[over].is_active() {
                            self.menu_states.move_to(over);
                            if let MenuEntry::SubMenu(submenu) = self.entries[over] {
                                if self.depth == self.menu_states.depth() {
                                    self.menu_states.push(submenu.profile());
                                }
                            }
                        } else {
                            self.menu_states.clear_active();
                        }
                    }
                    event::Status::Captured
                } else {
                    if self.menu_states.focused() == self.depth {
                        self.menu_states.clear_active()
                    }
                    event::Status::Ignored
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    if self.menu_states.focused() != self.depth {
                        self.menu_states.focus_on(self.depth);
                    }

                    let over = layout.children().position(|n| cursor.is_over(n.bounds()));
                    if let Some(over) = over {
                        if let MenuEntry::Item(_, _, Some(on_select)) = self.entries[over] {
                            shell.publish(on_select.clone());
                        }
                    }
                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }
}

/// The appearance of a [`Menu`].
#[derive(Clone, Copy, Debug)]
pub struct MenuAppearance {
    /// The background of a [`Menu`]
    pub background: Background,

    /// The fill of a separator in a [`Menu`]
    pub separator_fill: Background,

    /// The border radius of a [`Menu`]
    pub border_radius: f32,

    /// The border width of a [`Menu`]
    pub border_width: f32,

    /// The border color of a [`Menu`]
    pub border_color: Color,
}

/// The appearance of an item in a [`Menus`].
#[derive(Clone, Copy, Debug)]
pub struct MenuItemAppearance {
    /// The background of an item in a [`Menu`]
    pub background: Background,

    /// The text color of an item in a [`Menu`]
    pub text_color: Color,

    /// The text color of the keyboard shortcut for an item in a [`Menu`]
    pub shortcut_color: Color,

    /// The border radius of an item on a [`Menu`]
    pub border_radius: f32,

    /// The border width of an item on a [`Menu`]
    pub border_width: f32,

    /// The border color of an item on a [`Menu`]
    pub border_color: Color,
}