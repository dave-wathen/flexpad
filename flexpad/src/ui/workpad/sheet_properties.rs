use crate::{
    model::workpad::{Sheet, SheetId, WorkpadUpdate},
    ui::{
        action::{Action, ActionSet},
        labeled_element,
    },
};
use iced::{
    widget::{text, text_input},
    Command, Subscription,
};

use iced_aw::Card;
use rust_i18n::t;

use super::WorkpadMessage;

#[derive(Debug, Clone)]
pub enum SheetPropertiesMessage {
    Name(String),
    Finish(Action),
}

impl SheetPropertiesMessage {
    pub fn map_to_workpad(msg: SheetPropertiesMessage) -> WorkpadMessage {
        match msg {
            Self::Finish(Action::Ok) => WorkpadMessage::ModalSubmit,
            Self::Finish(Action::Cancel) => WorkpadMessage::ModalCancel,
            m => WorkpadMessage::SheetPropertiesMsg(m),
        }
    }
}

impl std::fmt::Display for SheetPropertiesMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SheetPropertiesMessage::")?;
        match self {
            Self::Name(name) => write!(f, "Name({name})"),
            Self::Finish(Action::Ok) => write!(f, "Finish(Submit)"),
            Self::Finish(Action::Cancel) => write!(f, "Finish(Cancel)"),
        }
    }
}

#[derive(Debug)]
pub struct SheetPropertiesUi {
    sheet_id: SheetId,
    name: String,
}

// TODO Focus management
impl SheetPropertiesUi {
    pub fn new(sheet: Sheet) -> Self {
        Self {
            sheet_id: sheet.id(),
            name: sheet.name().to_owned(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, SheetPropertiesMessage> {
        Card::new(
            text(t!("SheetProperties.Title")),
            labeled_element(
                t!("SheetName.Label"),
                text_input(&t!("SheetName.Placeholder"), &self.name)
                    .on_input(SheetPropertiesMessage::Name),
            ),
        )
        .foot(
            ActionSet::cancel_ok()
                .to_element()
                .map(SheetPropertiesMessage::Finish),
        )
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<SheetPropertiesMessage> {
        ActionSet::cancel_ok()
            .to_subscription()
            .map(SheetPropertiesMessage::Finish)
    }

    pub fn update(&mut self, message: SheetPropertiesMessage) -> Command<SheetPropertiesMessage> {
        match message {
            SheetPropertiesMessage::Name(name) => self.name = name,
            SheetPropertiesMessage::Finish(_) => unreachable!(),
        }
        Command::none()
    }

    pub fn into_update(self) -> WorkpadUpdate {
        WorkpadUpdate::SheetSetProperties {
            sheet_id: self.sheet_id,
            new_name: self.name,
        }
    }
}
