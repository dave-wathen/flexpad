use crate::ui::key::Key;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Element<Message>
where
    Message: Clone,
{
    Root(String),
    SubMenu(String),
    Section(String),
    Item(String, Option<Key>, Option<Message>),
}

/// A full path for a root menu to a menu item possibly via submenus and menu sections
pub struct Path<Message>
where
    Message: Clone,
{
    pub(super) elements: Vec<Element<Message>>,
}

impl<Message> Path<Message>
where
    Message: Clone,
{
    pub fn map<T, F>(self, mut f: F) -> Path<T>
    where
        F: Fn(Message) -> T,
        T: Clone,
    {
        Path {
            elements: self
                .elements
                .into_iter()
                .map(|e| match e {
                    Element::Root(name) => Element::Root(name),
                    Element::SubMenu(menu) => Element::SubMenu(menu),
                    Element::Section(name) => Element::Section(name),
                    Element::Item(name, shortcut, on_select) => {
                        Element::Item(name, shortcut, on_select.map(&mut f))
                    }
                })
                .collect(),
        }
    }
}

impl<Message> std::fmt::Display for Path<Message>
where
    Message: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.elements.iter() {
            match element {
                Element::Root(ref name) => write!(f, "{}", name)?,
                Element::SubMenu(name) => write!(f, " ... {}", name)?,
                Element::Section(name) => write!(f, "#{}", name)?,
                Element::Item(name, _, _) => write!(f, " ... {}", name)?,
            }
        }
        Ok(())
    }
}

/// A partial [`Path`] that terminates in a menu
pub struct PathToMenu<Message>
where
    Message: Clone,
{
    elements: Vec<Element<Message>>,
}

#[allow(dead_code)]
impl<Message> PathToMenu<Message>
where
    Message: Clone,
{
    /// Add a submenu to this partial path
    pub fn submenu(&self, name: impl ToString) -> PathToMenu<Message> {
        let name = name.to_string();
        assert!(!name.is_empty(), "Empty names not permitted for submenus");

        let mut elements = self.elements.clone();
        elements.push(Element::SubMenu(name));
        PathToMenu { elements }
    }

    /// Add a menu section to this partial path.  Menu sections are separated
    /// visually within a menu.  A menu will consist of the items added without
    /// a section, followed by a separator and the items for each section.
    /// Sections are ordered by their names.
    pub fn section(&self, name: impl ToString) -> PathToMenuSection<Message> {
        let name = name.to_string();
        assert!(!name.is_empty(), "Empty names not permitted for sections");

        let mut elements = self.elements.clone();
        elements.push(Element::Section(name));
        PathToMenuSection { elements }
    }

    /// Add a menu item to this partial path to give a full [`Path`].
    pub fn item(&self, item: PathItem<Message>) -> Path<Message> {
        let mut elements = self.elements.clone();
        elements.push(Element::Item(item.name, item.shortcut, item.on_select));
        Path { elements }
    }
}

/// A partial [`Path`] that terminates in a menu section
#[allow(dead_code)]
pub struct PathToMenuSection<Message>
where
    Message: Clone,
{
    elements: Vec<Element<Message>>,
}
#[allow(dead_code)]
impl<Message> PathToMenuSection<Message>
where
    Message: Clone,
{
    /// Add a submenu to this partial path
    pub fn submenu(&self, name: impl ToString) -> PathToMenu<Message> {
        let name = name.to_string();
        assert!(!name.is_empty(), "Empty names not permitted for submenus");

        let mut elements = self.elements.clone();
        elements.push(Element::SubMenu(name));
        PathToMenu { elements }
    }

    pub fn item(&self, item: PathItem<Message>) -> Path<Message> {
        let mut elements = self.elements.clone();
        elements.push(Element::Item(item.name, item.shortcut, item.on_select));
        Path { elements }
    }
}

/// A menu item that will terminate a ['Path']
pub struct PathItem<Message> {
    name: String,
    shortcut: Option<Key>,
    on_select: Option<Message>,
}

impl<Message> PathItem<Message> {
    /// Add a shortcut key for this item
    pub fn shortcut(self, key: Key) -> Self {
        Self {
            shortcut: Some(key),
            ..self
        }
    }

    /// Add a message to be emitted when this item is selected (via the menu or its shortcut)
    pub fn on_select(self, message: Message) -> Self {
        Self {
            on_select: Some(message),
            ..self
        }
    }
}

/// Create a new partial path from a root menu.  A root menu is one whose name will
/// appear in the menu bar.
pub fn root<Message>(name: impl ToString) -> PathToMenu<Message>
where
    Message: Clone,
{
    let name = name.to_string();
    assert!(!name.is_empty(), "Empty names not permitted for root menus");

    PathToMenu {
        elements: vec![Element::Root(name)],
    }
}

/// Create a menu item that can be used to complete a partial path.
pub fn item<Message>(name: impl ToString) -> PathItem<Message>
where
    Message: Clone,
{
    let name = name.to_string();
    assert!(!name.is_empty(), "Empty names not permitted for menu items");

    PathItem {
        name,
        shortcut: None,
        on_select: None,
    }
}
