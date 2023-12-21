use std::rc::Rc;

use iced::{
    advanced::{
        self,
        layout::{self, Node},
        mouse, overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    event, keyboard,
    widget::vertical_space,
    Element, Event, Length, Padding, Pixels, Point, Rectangle, Size,
};

use crate::ui::util::{
    key::{ctrl, key, pressed, released, Key},
    SPACE_M, SPACE_S,
};

mod bar;
use bar::MenuBar;
pub use bar::{MenuBarAppearance, MenuBarItemAppearance};

mod data;
use data::{MenuEntry, Roots};

mod menu_overlay;
use menu_overlay::MenuOverlay;
pub use menu_overlay::{MenuAppearance, MenuItemAppearance};

mod path;
pub use path::{item, root, PartialPath, Path, PathItem, PathToMenu, PathToMenuSection, PathVec};

mod state;
use state::MenuStates;

const SEPARATOR_HEIGHT: f32 = 1.0;

const CTRL_F2: keyboard::Event = released(ctrl(key(keyboard::KeyCode::F2)));
const ESCAPE: keyboard::Event = pressed(key(keyboard::KeyCode::Escape));
const ENTER: keyboard::Event = pressed(key(keyboard::KeyCode::Enter));
const LEFT: keyboard::Event = pressed(key(keyboard::KeyCode::Left));
const RIGHT: keyboard::Event = pressed(key(keyboard::KeyCode::Right));
const UP: keyboard::Event = pressed(key(keyboard::KeyCode::Up));
const DOWN: keyboard::Event = pressed(key(keyboard::KeyCode::Down));

const MENUBAR_PADDING: Padding = Padding::new(SPACE_S);
const MENUBAR_ENTRY_PADDING: Padding = Padding {
    top: SPACE_S,
    right: SPACE_M,
    bottom: SPACE_S,
    left: SPACE_M,
};
const MENU_PADDING: Padding = Padding::new(SPACE_S);
const MENU_ENTRY_PADDING: Padding = Padding {
    top: SPACE_S,
    right: SPACE_M,
    bottom: SPACE_S,
    left: SPACE_M,
};
const TEXT_SIZE_MENU: Pixels = Pixels(14.0);

/// A limited emulation of macOS menus to suffice until system menu support
/// is available in Iced.
pub struct MenuedContent<'a, Message, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    roots: Rc<Roots<Message>>,
    menu_bar: Element<'a, Message, Renderer>,
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> MenuedContent<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a [`MenuedContent`] with the given content.
    pub fn new<T>(paths: PathVec<Message>, content: T) -> Self
    where
        Message: 'a,
        Renderer: 'a,
        T: Into<Element<'a, Message, Renderer>>,
    {
        let roots = Rc::new(Roots::new(paths));

        let menu_bar = match roots.is_empty() {
            true => vertical_space(0.0).into(),
            false => MenuBar::new(Rc::clone(&roots)).into(),
        };

        Self {
            width: Length::Fill,
            height: Length::Fill,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            roots,
            menu_bar,
            content: content.into(),
        }
    }

    /// Sets the width of the [`MenuedContent`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`MenuedContent`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`MenuedContent`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`MenuedContent`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for MenuedContent<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.menu_bar), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > 2 {
            tree.children.truncate(2);
        }

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&self.menu_bar));
        } else {
            tree.children[0].diff(&self.menu_bar)
        }

        if tree.children.len() < 2 {
            tree.children.push(Tree::new(&self.content));
        } else {
            tree.children[1].diff(&self.content)
        }
    }

    fn width(&self) -> Length {
        self.width
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
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        let mb_layout = self
            .menu_bar
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits);
        let mb_size = mb_layout.size();

        let content_limits = limits.shrink(Size::new(0.0, mb_size.height));
        let mut c_layout =
            self.content
                .as_widget()
                .layout(&mut tree.children[1], renderer, &content_limits);
        c_layout.move_to(Point::new(0.0, mb_size.height));
        let c_size = c_layout.size();

        let size = Size::new(
            mb_size.width.max(c_size.width),
            mb_size.height + c_size.height,
        );
        Node::with_children(limits.resolve(size), vec![mb_layout, c_layout])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            [self.menu_bar.as_widget(), self.content.as_widget()]
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
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key_code,
            modifiers,
        }) = event
        {
            let key = Key::new(modifiers, key_code);
            if let Some(message) = self.roots.for_shortcut(key) {
                shell.publish(message);
                return event::Status::Captured;
            }
        }

        let mut child_layouts = layout.children();
        let mut child_states = tree.children.iter_mut();

        let status = self.menu_bar.as_widget_mut().on_event(
            child_states.next().unwrap(),
            event.clone(),
            child_layouts.next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if status == event::Status::Captured {
            status
        } else {
            self.content.as_widget_mut().on_event(
                child_states.next().unwrap(),
                event,
                child_layouts.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        [self.menu_bar.as_widget(), self.content.as_widget()]
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
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as advanced::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in [self.menu_bar.as_widget(), self.content.as_widget()]
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.draw(state, renderer, theme, style, layout, cursor, viewport)
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let mut child_layouts = layout.children();
        let mut child_states = tree.children.iter_mut();

        let mb_overlay = self.menu_bar.as_widget_mut().overlay(
            child_states.next().unwrap(),
            child_layouts.next().unwrap(),
            renderer,
        );

        let c_overlay = self.content.as_widget_mut().overlay(
            child_states.next().unwrap(),
            child_layouts.next().unwrap(),
            renderer,
        );

        match (mb_overlay, c_overlay) {
            (None, None) => None,
            (None, Some(c)) => Some(c),
            (Some(mb), None) => Some(mb),
            (Some(mb), Some(c)) => Some(overlay::Group::with_children(vec![mb, c]).into()),
        }
    }
}

impl<'a, Message, Renderer> From<MenuedContent<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer: iced::advanced::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(menu: MenuedContent<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(menu)
    }
}

/// StyleSheet to deterrmine the appearance of the [`MenuBar`] and [`Menus`]
pub trait StyleSheet {
    type Style: Default;
    /// The normal appearance of a [`MenuBar`]
    fn menu_bar(&self, style: &Self::Style) -> MenuBarAppearance;

    /// The appearance of an item in a [`MenuBar`] when it is active but not selected
    fn active_menu_bar_item(&self, style: &Self::Style) -> MenuBarItemAppearance;

    /// The appearance of an item in a [`MenuBar`] when it is active but not selected
    fn selected_menu_bar_item(&self, style: &Self::Style) -> MenuBarItemAppearance;

    /// The appearance of a [`Menu`]
    fn menu(&self, style: &Self::Style) -> MenuAppearance;

    /// The appearance of an item in a [`Menu`] when it is inactive (has no on_select)
    fn inactive_menu_item(&self, style: &Self::Style) -> MenuItemAppearance;

    /// The appearance of an item in a [`Menu`] when it is active (has an on_select)
    fn active_menu_item(&self, style: &Self::Style) -> MenuItemAppearance;

    /// The appearance of an item in a [`Menu`] when it is active and selected in a menu that has focus
    fn focused_selected_menu_item(&self, style: &Self::Style) -> MenuItemAppearance;

    /// The appearance of an item in a [`Menu`] when it is active and selected in a menu that does not have focus
    fn unfocused_selected_menu_item(&self, style: &Self::Style) -> MenuItemAppearance;
}
