use crate::ui::util::{
    dialog_button, dialog_title, handle_cancel_key, handle_ok_and_cancel_keys, text_input,
};
use flexpad_model::{Sheet, WorkpadMaster, WorkpadUpdate};
use flexpad_toolkit::{button_bar::ButtonBar, dialog::Dialog, prelude::*};
use iced::{event, widget::column, Subscription};
use rust_i18n::t;

#[derive(Debug, Clone)]
pub enum Message {
    Name(String),
    Cancel,
    Submit,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SheetPropertiesMessage::")?;
        match self {
            Self::Name(name) => write!(f, "Name({name})"),
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
pub struct SheetPropertiesUi {
    sheet: Sheet,
    other_names: Vec<String>,
    name: String,
    name_error: Option<String>,
}

// TODO Focus management
impl SheetPropertiesUi {
    pub fn new(sheet: Sheet) -> Self {
        let other_names = sheet
            .workpad()
            .sheets()
            .filter(|s| *s != sheet)
            .map(|s| s.name().to_owned())
            .collect();
        let name = sheet.name().to_owned();

        Self {
            sheet,
            other_names,
            name,
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
                t!("SheetName.Label"),
                t!("SheetName.Placeholder"),
                &self.name,
                Message::Name,
                self.name_error.as_ref(),
            ),
            ButtonBar::new().push(cancel).push(ok)
        ]
        .spacing(SPACE_S);

        Dialog::new(
            dialog_title(t!("SheetProperties.Title"), Default::default()),
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
                if self.other_names.contains(&name) {
                    self.name_error = Some(t!("SheetName.AlreadyUsedError"))
                } else if name.is_empty() {
                    self.name_error = Some(t!("SheetName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = name;
                Event::None
            }
            Message::Cancel => Event::Cancelled,
            Message::Submit => Event::Submitted(
                self.sheet.workpad().master(),
                WorkpadUpdate::SheetSetProperties {
                    sheet_id: self.sheet.id(),
                    new_name: self.name.clone(),
                },
            ),
        }
    }
}
