use crate::{
    model::workpad::{Sheet, SheetId, WorkpadUpdate},
    ui::{ok_cancel::OkCancel, SPACE_S},
};
use iced::{
    widget::{column, text, text_input},
    Command, Subscription,
};

use iced_aw::Card;

#[derive(Debug, Clone)]
pub enum SheetPropertiesMessage {
    Name(String),
    Finish(OkCancel),
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
            text("Workpad Properties"),
            column![
                text("Name").size(12),
                text_input("Workpad Name", &self.name,)
                    .on_input(SheetPropertiesMessage::Name)
                    .padding(5),
            ]
            .spacing(SPACE_S),
        )
        .foot(OkCancel::buttons_view().map(SheetPropertiesMessage::Finish))
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<SheetPropertiesMessage> {
        OkCancel::subscription().map(SheetPropertiesMessage::Finish)
    }

    pub fn update(&mut self, message: SheetPropertiesMessage) -> Command<SheetPropertiesMessage> {
        match message {
            SheetPropertiesMessage::Name(name) => self.name = name,
            SheetPropertiesMessage::Finish(_) => {}
        }
        Command::none()
    }

    pub fn into_update(self) -> WorkpadUpdate {
        WorkpadUpdate::SetSheetProperties {
            sheet_id: self.sheet_id,
            new_name: self.name,
        }
    }
}
