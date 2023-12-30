use crate::ui::{
    style,
    util::{
        button_bar, dialog::Dialog, dialog_button, dialog_title, handle_cancel_key,
        handle_ok_and_cancel_keys, text_input, SPACE_S,
    },
};
use flexpad_model::{Workpad, WorkpadMaster, WorkpadUpdate};
use iced::{event, widget::column, Subscription};
use rust_i18n::t;

#[derive(Debug, Clone)]
pub enum Message {
    Name(String),
    Author(String),
    Submit,
    Cancel,
}

impl std::fmt::Display for Message {
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

pub enum Event {
    None,
    Cancelled,
    Submitted(WorkpadMaster, WorkpadUpdate),
}

#[derive(Debug)]
pub struct PadPropertiesUi {
    pad: Workpad,
    name: String,
    author: String,
    name_error: Option<String>,
}

// TODO Focus management
impl PadPropertiesUi {
    pub fn new(pad: Workpad) -> Self {
        let name = pad.name().to_owned();
        let author = pad.author().to_owned();
        Self {
            pad,
            name,
            author,
            name_error: None,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let cancel = dialog_button(t!("Common.Cancel"), style::DialogButtonStyle::Cancel)
            .on_press(Message::Cancel);

        let mut ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Ok);
        if self.name_error.is_none() {
            ok = ok.on_press(Message::Submit)
        }

        let body = column![
            text_input(
                t!("PadName.Label"),
                t!("PadName.Placeholder"),
                &self.name,
                Message::Name,
                self.name_error.as_ref(),
            ),
            text_input(
                t!("PadAuthor.Label"),
                t!("PadAuthor.Placeholder"),
                &self.author,
                Message::Author,
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

    pub fn subscription(&self) -> Subscription<Message> {
        if self.name_error.is_none() {
            event::listen_with(|event, _status| {
                handle_ok_and_cancel_keys(&event, Message::Submit, Message::Cancel)
            })
        } else {
            event::listen_with(|event, _status| handle_cancel_key(&event, Message::Cancel))
        }
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Name(name) => {
                if name.is_empty() {
                    self.name_error = Some(t!("PadName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = name;
                Event::None
            }
            Message::Author(author) => {
                self.author = author;
                Event::None
            }
            Message::Cancel => Event::Cancelled,
            Message::Submit => Event::Submitted(
                self.pad.master(),
                WorkpadUpdate::WorkpadSetProperties {
                    new_name: self.name.clone(),
                    new_author: self.author.clone(),
                },
            ),
        }
    }
}
