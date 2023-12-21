use crate::ui::util::action::Action;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Element<Message>
where
    Message: Clone,
{
    Root(String),
    SubMenu(String),
    Section(String),
    Action(Action, Option<Message>),
}

/// A collection of [`Path`]s
pub struct PathVec<Message>
where
    Message: Clone,
{
    pub(super) paths: Vec<Path<Message>>,
}

impl<Message> PathVec<Message>
where
    Message: Clone,
{
    pub fn new() -> Self {
        Self { paths: vec![] }
    }

    pub fn with(mut self, path: Path<Message>) -> Self {
        self.paths.push(path);
        self
    }

    pub fn extend(mut self, more: PathVec<Message>) -> Self {
        self.paths.extend(more.paths);
        self
    }

    pub fn map<T, F>(self, f: F) -> PathVec<T>
    where
        F: Fn(Message) -> T,
        T: Clone,
    {
        PathVec {
            paths: self.paths.into_iter().map(|p| p.map(&f)).collect(),
        }
    }
}

impl<Message> Default for PathVec<Message>
where
    Message: Clone,
{
    fn default() -> Self {
        Self::new()
    }
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
    pub fn new(
        partial: impl PartialPath<Message>,
        action: &Action,
        on_select: Option<Message>,
    ) -> Path<Message>
    where
        Message: Clone,
    {
        let action = menu_action(action);
        match on_select {
            Some(msg) => partial.action(action.on_select(msg)),
            None => partial.action(action),
        }
    }

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
                    Element::Action(action, on_select) => {
                        Element::Action(action, on_select.map(&mut f))
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
                Element::Action(action, _) => write!(f, " ... {}", action.name)?,
            }
        }
        Ok(())
    }
}

pub trait PartialPath<Message>
where
    Message: Clone,
{
    /// Add an action to this partial path to give a full [`Path`].
    fn action(&self, action: PathAction<Message>) -> Path<Message>;
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
    /// visually within a menu.  A menu will consist of the actions added without
    /// a section, followed by a separator and the actions for each section.
    /// Sections are ordered by their names.
    pub fn section(&self, name: impl ToString) -> PathToMenuSection<Message> {
        let name = name.to_string();
        assert!(!name.is_empty(), "Empty names not permitted for sections");

        let mut elements = self.elements.clone();
        elements.push(Element::Section(name));
        PathToMenuSection { elements }
    }
}

impl<Message> PartialPath<Message> for PathToMenu<Message>
where
    Message: Clone,
{
    fn action(&self, action: PathAction<Message>) -> Path<Message> {
        let mut elements = self.elements.clone();
        elements.push(Element::Action(action.action, action.on_select));
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
}

impl<Message> PartialPath<Message> for PathToMenuSection<Message>
where
    Message: Clone,
{
    fn action(&self, action: PathAction<Message>) -> Path<Message> {
        let mut elements = self.elements.clone();
        elements.push(Element::Action(action.action, action.on_select));
        Path { elements }
    }
}

/// A menu action that will terminate a ['Path']
pub struct PathAction<Message> {
    action: Action,
    on_select: Option<Message>,
}

impl<Message> PathAction<Message> {
    /// Add a message to be emitted when this action is selected (via the menu or its shortcut)
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

/// Create a menu action that can be used to complete a partial path.
pub fn menu_action<Message>(action: &Action) -> PathAction<Message>
where
    Message: Clone,
{
    PathAction {
        action: action.clone(),
        on_select: None,
    }
}
