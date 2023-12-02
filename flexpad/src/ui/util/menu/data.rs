use crate::ui::util::key::Key;

use super::{path, PathVec};

pub(super) enum MenuEntry<Message>
where
    Message: Clone,
{
    SectionStart(String),
    SubMenu(Menu<Message>),
    Item(String, Option<Key>, Option<Message>),
}

impl<Message> MenuEntry<Message>
where
    Message: Clone,
{
    pub(super) fn is_active(&self) -> bool {
        match self {
            Self::SectionStart(_) => false,
            Self::Item(_, _, on_select) => on_select.is_some(),
            Self::SubMenu(_) => true,
        }
    }
}

pub(super) struct Roots<Message>
where
    Message: Clone,
{
    roots: Vec<Menu<Message>>,
}

impl<Message> Roots<Message>
where
    Message: Clone,
{
    pub(super) fn new(paths: PathVec<Message>) -> Self {
        let mut bar = Self { roots: vec![] };

        for path in paths.paths {
            bar.push_path_iter(path.elements.into_iter());
        }

        bar
    }

    fn push_path_iter(&mut self, mut path: impl Iterator<Item = path::Element<Message>>) {
        match path.next() {
            Some(path::Element::Root(root_name)) => {
                match self.roots.iter_mut().find(|m| m.name == root_name) {
                    Some(menu) => menu.push_path_iter(path),
                    None => {
                        let mut menu = Menu::new(root_name);
                        menu.push_path_iter(path);
                        self.roots.push(menu);
                    }
                };
            }
            _ => panic!("Path should begin with root"),
        }
    }

    /// Find the message for a given shortcut key (if any)
    pub(super) fn for_shortcut(&self, key: Key) -> Option<Message> {
        self.roots.iter().find_map(|m| m.for_shortcut(key))
    }

    pub(super) fn len(&self) -> usize {
        self.roots.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &Menu<Message>> {
        self.roots.iter()
    }
}

impl<Message> std::ops::Index<usize> for Roots<Message>
where
    Message: Clone,
{
    type Output = Menu<Message>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.roots[index]
    }
}

pub(super) struct Menu<Message>
where
    Message: Clone,
{
    pub(super) name: String,
    entries: Vec<MenuEntry<Message>>,
}

impl<Message> Menu<Message>
where
    Message: Clone,
{
    pub(super) fn new(title: impl ToString) -> Self {
        Self {
            name: title.to_string(),
            entries: vec![],
        }
    }

    pub(super) fn push_path_iter(
        &mut self,
        mut path: impl Iterator<Item = path::Element<Message>>,
    ) {
        let next = path.next().expect("End of path not expected");

        let (section, next) = if let path::Element::Section(name) = next {
            (name, path.next().expect("End of path not expected"))
        } else {
            (String::new(), next)
        };

        let section_start = self
            .entries
            .iter()
            .position(|e| matches!(e, MenuEntry::SectionStart(name) if name == &section));

        let section_start = match section_start {
            Some(index) => index,
            None => {
                let start = MenuEntry::SectionStart(section.to_string());
                let before = self
                    .entries
                    .iter()
                    .position(|e| matches!(e, MenuEntry::SectionStart(name) if name > &section));
                match before {
                    Some(index) => {
                        self.entries.insert(index, start);
                        index
                    }
                    None => {
                        self.entries.push(start);
                        self.entries.len() - 1
                    }
                }
            }
        };

        let section_end = self
            .entries
            .iter()
            .position(|e| matches!(e, MenuEntry::SectionStart(name) if name > &section))
            .unwrap_or(self.entries.len());

        if let path::Element::SubMenu(name) = next {
            let existing = self.entries[(section_start + 1)..section_end]
                .iter_mut()
                .find(|e| matches!(e, MenuEntry::SubMenu(menu) if menu.name == name));
            if let Some(MenuEntry::SubMenu(ref mut menu)) = existing {
                menu.push_path_iter(path)
            } else {
                let mut menu = Menu::new(name);
                menu.push_path_iter(path);
                self.entries.insert(section_end, MenuEntry::SubMenu(menu));
            }
        } else if let path::Element::Item(name, shortcut, on_select) = next {
            self.entries
                .insert(section_end, MenuEntry::Item(name, shortcut, on_select));
        }
    }

    fn for_shortcut(&self, key: Key) -> Option<Message> {
        self.entries
            .iter()
            .find_map(|e| match e {
                MenuEntry::Item(_, Some(shortcut), Some(on_select)) if *shortcut == key => {
                    Some(on_select.clone())
                }
                _ => None,
            })
            .or_else(|| {
                self.entries.iter().find_map(|e| match e {
                    MenuEntry::SubMenu(menu) => menu.for_shortcut(key),
                    _ => None,
                })
            })
    }

    // The profile is the shape of the menu.  It's a Vec<bool> denoting whether each element in `entries()` is selectable.
    pub(super) fn profile(&self) -> Vec<bool> {
        self.entries()
            .iter()
            .cloned()
            .map(MenuEntry::is_active)
            .collect()
    }

    // Returns the menus entries skipping the first section start
    pub(super) fn entries(&self) -> Vec<&MenuEntry<Message>> {
        self.entries.iter().skip(1).collect()
    }
}
