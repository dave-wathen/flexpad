use crate::{
    model::workpad::{Workpad, WorkpadUpdate},
    ui::{
        action::{Action, ActionSet},
        labeled_element, SPACE_M,
    },
};
use iced::{
    widget::{column, text, text_input},
    Command, Subscription,
};
use iced_aw::Card;
use rust_i18n::t;

use super::WorkpadMessage;

#[derive(Debug, Clone)]
pub enum PadPropertiesMessage {
    Name(String),
    Author(String),
    Finish(Action),
}

impl PadPropertiesMessage {
    pub fn map_to_workpad(msg: PadPropertiesMessage) -> WorkpadMessage {
        match msg {
            Self::Finish(Action::Ok) => WorkpadMessage::ModalSubmit,
            Self::Finish(Action::Cancel) => WorkpadMessage::ModalCancel,
            m => WorkpadMessage::PadPropertiesMsg(m),
        }
    }
}

impl std::fmt::Display for PadPropertiesMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PadPropertiesMessage::")?;
        match self {
            Self::Name(name) => write!(f, "Name({name})"),
            Self::Author(author) => write!(f, "Author({author})"),
            Self::Finish(Action::Ok) => write!(f, "Finish(Submit)"),
            Self::Finish(Action::Cancel) => write!(f, "Finish(Cancel)"),
        }
    }
}

#[derive(Debug)]
pub struct PadPropertiesUi {
    name: String,
    author: String,
}

// TODO Focus management
impl PadPropertiesUi {
    pub fn new(pad: Workpad) -> Self {
        Self {
            name: pad.name().to_owned(),
            author: pad.author().to_owned(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, PadPropertiesMessage> {
        Card::new(
            text(t!("PadProperties.Title")),
            column![
                labeled_element(
                    t!("PadName.Label"),
                    text_input(&t!("Forms.PadName.Placeholder"), &self.name)
                        .on_input(PadPropertiesMessage::Name)
                ),
                labeled_element(
                    t!("PadAuthor.Label"),
                    text_input(&t!("Forms.PadAuthor.Placeholder"), &self.author)
                        .on_input(PadPropertiesMessage::Author)
                ),
            ]
            .spacing(SPACE_M),
        )
        .foot(
            ActionSet::cancel_ok()
                .to_element()
                .map(PadPropertiesMessage::Finish),
        )
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<PadPropertiesMessage> {
        ActionSet::cancel_ok()
            .to_subscription()
            .map(PadPropertiesMessage::Finish)
    }

    pub fn update(&mut self, message: PadPropertiesMessage) -> Command<PadPropertiesMessage> {
        match message {
            PadPropertiesMessage::Name(name) => self.name = name,
            PadPropertiesMessage::Author(author) => self.author = author,
            PadPropertiesMessage::Finish(_) => {}
        }
        Command::none()
    }

    pub fn into_update(self) -> WorkpadUpdate {
        WorkpadUpdate::WorkpadSetProperties {
            new_name: self.name,
            new_author: self.author,
        }
    }
}
