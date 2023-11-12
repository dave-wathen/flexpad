use crate::{
    model::workpad::{Sheet, SheetId, WorkpadUpdate},
    ui::{
        button_bar, dialog::Dialog, dialog_button, dialog_title, handle_cancel_key,
        handle_ok_and_cancel_keys, style, text_input, SPACE_S,
    },
};
use iced::{subscription, widget::column, Command, Subscription};

use rust_i18n::t;

use super::WorkpadMessage;

#[derive(Debug, Clone)]
pub enum SheetPropertiesMessage {
    Name(String),
    Cancel,
    Submit,
}

impl SheetPropertiesMessage {
    pub fn map_to_workpad(msg: SheetPropertiesMessage) -> WorkpadMessage {
        match msg {
            Self::Submit => WorkpadMessage::ModalSubmit,
            Self::Cancel => WorkpadMessage::ModalCancel,
            m => WorkpadMessage::SheetPropertiesMsg(m),
        }
    }
}

impl std::fmt::Display for SheetPropertiesMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SheetPropertiesMessage::")?;
        match self {
            Self::Name(name) => write!(f, "Name({name})"),
            Self::Cancel => write!(f, "Cancel"),
            Self::Submit => write!(f, "Submit"),
        }
    }
}

#[derive(Debug)]
pub struct SheetPropertiesUi {
    sheet_id: SheetId,
    other_names: Vec<String>,
    name: String,
    name_error: Option<String>,
}

// TODO Focus management
impl SheetPropertiesUi {
    pub fn new(sheet: Sheet) -> Self {
        Self {
            sheet_id: sheet.id(),
            other_names: sheet
                .workpad()
                .sheets()
                .filter(|s| *s != sheet)
                .map(|s| s.name().to_owned())
                .collect(),
            name: sheet.name().to_owned(),
            name_error: None,
        }
    }

    pub fn view(&self) -> iced::Element<'_, SheetPropertiesMessage> {
        let cancel = dialog_button(t!("Common.Cancel"), style::DialogButtonStyle::Cancel)
            .on_press(SheetPropertiesMessage::Cancel);

        let mut ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Ok);
        if self.name_error.is_none() {
            ok = ok.on_press(SheetPropertiesMessage::Submit)
        }

        let body = column![
            text_input(
                t!("SheetName.Label"),
                t!("SheetName.Placeholder"),
                &self.name,
                SheetPropertiesMessage::Name,
                self.name_error.as_ref(),
            ),
            button_bar().push(cancel).push(ok)
        ]
        .spacing(SPACE_S);

        Dialog::new(
            dialog_title(t!("SheetProperties.Title"), Default::default()),
            body,
        )
        .max_width(400.0)
        .into()
    }

    pub fn subscription(&self) -> Subscription<SheetPropertiesMessage> {
        if self.name_error.is_none() {
            subscription::events_with(|event, _status| {
                handle_ok_and_cancel_keys(
                    &event,
                    SheetPropertiesMessage::Submit,
                    SheetPropertiesMessage::Cancel,
                )
            })
        } else {
            subscription::events_with(|event, _status| {
                handle_cancel_key(&event, SheetPropertiesMessage::Cancel)
            })
        }
    }

    pub fn update(&mut self, message: SheetPropertiesMessage) -> Command<SheetPropertiesMessage> {
        match message {
            SheetPropertiesMessage::Name(name) => {
                if self.other_names.contains(&name) {
                    self.name_error = Some(t!("SheetName.AlreadyUsedError"))
                } else if name.is_empty() {
                    self.name_error = Some(t!("SheetName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = name
            }
            _ => unreachable!(),
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
