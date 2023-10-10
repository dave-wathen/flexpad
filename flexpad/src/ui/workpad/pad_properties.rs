use crate::{
    model::workpad::{Workpad, WorkpadUpdate},
    ui::{ok_cancel::OkCancel, SPACE_M, SPACE_S},
};
use iced::{
    widget::{column, text, text_input},
    Command, Subscription,
};
use iced_aw::Card;

#[derive(Debug, Clone)]
pub enum PadPropertiesMessage {
    Name(String),
    Author(String),
    Finish(OkCancel),
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
            text("Workpad Properties"),
            column![
                column![
                    text("Name").size(12),
                    text_input("Workpad Name", &self.name,)
                        .on_input(PadPropertiesMessage::Name)
                        .padding(5),
                ]
                .spacing(SPACE_S),
                column![
                    text("Author").size(12),
                    text_input("", &self.author)
                        .on_input(PadPropertiesMessage::Author)
                        .padding(5),
                ]
                .spacing(SPACE_S)
            ]
            .spacing(SPACE_M),
        )
        .foot(OkCancel::buttons_view().map(PadPropertiesMessage::Finish))
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<PadPropertiesMessage> {
        OkCancel::subscription().map(PadPropertiesMessage::Finish)
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
        WorkpadUpdate::SetWorkpadProperties {
            new_name: self.name,
            new_author: self.author,
        }
    }
}
