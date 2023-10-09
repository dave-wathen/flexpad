use crate::{
    model::workpad::{Workpad, WorkpadUpdate},
    ui::{DIALOG_BUTTON_WIDTH, SPACE_M, SPACE_S},
};
use iced::{
    alignment::Horizontal,
    theme,
    widget::{button, column, horizontal_space, row, text, text_input},
    Command, Length,
};
use iced_aw::Card;

#[derive(Debug, Clone)]
pub enum PadPropertiesMessage {
    Name(String),
    Author(String),
    Cancel,
    Submit,
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
                        .on_submit(PadPropertiesMessage::Submit)
                        .padding(5),
                ]
                .spacing(SPACE_S),
                column![
                    text("Author").size(12),
                    text_input("", &self.author)
                        .on_input(PadPropertiesMessage::Author)
                        .on_submit(PadPropertiesMessage::Submit)
                        .padding(5),
                ]
                .spacing(SPACE_S)
            ]
            .spacing(SPACE_M),
        )
        .foot(
            row![
                horizontal_space(Length::Fill),
                row![
                    button(text("Cancel").horizontal_alignment(Horizontal::Center))
                        .width(DIALOG_BUTTON_WIDTH)
                        .style(theme::Button::Secondary)
                        .on_press(PadPropertiesMessage::Cancel),
                    button(text("Ok").horizontal_alignment(Horizontal::Center))
                        .width(DIALOG_BUTTON_WIDTH)
                        .on_press(PadPropertiesMessage::Submit),
                ]
                .spacing(SPACE_S)
            ]
            .width(Length::Fill),
        )
        .max_width(400.0)
        .into()
    }

    pub fn update(&mut self, message: PadPropertiesMessage) -> Command<PadPropertiesMessage> {
        match message {
            PadPropertiesMessage::Name(name) => self.name = name,
            PadPropertiesMessage::Author(author) => self.author = author,
            PadPropertiesMessage::Submit | PadPropertiesMessage::Cancel => {}
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
