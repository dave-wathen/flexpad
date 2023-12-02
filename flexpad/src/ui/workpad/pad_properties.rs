use crate::{
    model::workpad::{Workpad, WorkpadUpdate},
    ui::{
        style,
        util::{
            button_bar, dialog::Dialog, dialog_button, dialog_title, handle_cancel_key,
            handle_ok_and_cancel_keys, text_input, SPACE_S,
        },
    },
};
use iced::{subscription, widget::column, Command, Subscription};
use rust_i18n::t;

use super::WorkpadMessage;

#[derive(Debug, Clone)]
pub enum PadPropertiesMessage {
    Name(String),
    Author(String),
    Submit,
    Cancel,
}

impl PadPropertiesMessage {
    pub fn map_to_workpad(msg: PadPropertiesMessage) -> WorkpadMessage {
        match msg {
            Self::Submit => WorkpadMessage::ModalSubmit,
            Self::Cancel => WorkpadMessage::ModalCancel,
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
            Self::Cancel => write!(f, "Cancel"),
            Self::Submit => write!(f, "Submit"),
        }
    }
}

#[derive(Debug)]
pub struct PadPropertiesUi {
    name: String,
    author: String,
    name_error: Option<String>,
}

// TODO Focus management
impl PadPropertiesUi {
    pub fn new(pad: Workpad) -> Self {
        Self {
            name: pad.name().to_owned(),
            author: pad.author().to_owned(),
            name_error: None,
        }
    }

    pub fn view(&self) -> iced::Element<'_, PadPropertiesMessage> {
        let cancel = dialog_button(t!("Common.Cancel"), style::DialogButtonStyle::Cancel)
            .on_press(PadPropertiesMessage::Cancel);

        let mut ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Ok);
        if self.name_error.is_none() {
            ok = ok.on_press(PadPropertiesMessage::Submit)
        }

        let body = column![
            text_input(
                t!("PadName.Label"),
                t!("PadName.Placeholder"),
                &self.name,
                PadPropertiesMessage::Name,
                self.name_error.as_ref(),
            ),
            text_input(
                t!("PadAuthor.Label"),
                t!("PadAuthor.Placeholder"),
                &self.author,
                PadPropertiesMessage::Author,
                None,
            ),
            button_bar().push(cancel).push(ok)
        ]
        .spacing(SPACE_S);

        Dialog::new(
            dialog_title(t!("PadProperties.Title"), Default::default()),
            body,
        )
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<PadPropertiesMessage> {
        if self.name_error.is_none() {
            subscription::events_with(|event, _status| {
                handle_ok_and_cancel_keys(
                    &event,
                    PadPropertiesMessage::Submit,
                    PadPropertiesMessage::Cancel,
                )
            })
        } else {
            subscription::events_with(|event, _status| {
                handle_cancel_key(&event, PadPropertiesMessage::Cancel)
            })
        }
    }

    pub fn update(&mut self, message: PadPropertiesMessage) -> Command<PadPropertiesMessage> {
        match message {
            PadPropertiesMessage::Name(name) => {
                if name.is_empty() {
                    self.name_error = Some(t!("PadName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = name;
            }
            PadPropertiesMessage::Author(author) => self.author = author,
            _ => unreachable!(),
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
