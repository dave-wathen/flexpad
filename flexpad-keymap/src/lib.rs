use std::collections::HashMap;

use iced::{advanced::widget, Element};

type Renderer<Theme = iced::Theme> = iced::Renderer<Theme>;

pub struct KeyMap<Message> {
    map: HashMap<KeyMapping, Message>,
}

pub struct KeyMapping {
    // TODO
}

pub struct KeyMapContainer<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    id: Option<Id>,
    content: Element<'a, Message, Renderer>,
}

// TODO Implement

/// The identifier of a [`Container`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
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
