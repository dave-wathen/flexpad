use crate::{
    model::workpad::{SheetKind, UpdateError, Workpad, WorkpadUpdate},
    ui::{
        menu, style,
        util::{
            button_bar, dialog_button, handle_ok_and_cancel_keys, handle_ok_key, images,
            text_input, SPACE_M, SPACE_S,
        },
    },
};
use iced::{
    alignment, subscription,
    theme::{self},
    widget::{button, column, container, horizontal_rule, image, row, text, vertical_space},
    Alignment, Command, Element, Length, Subscription,
};
use rust_i18n::t;
use tracing::debug;

use super::WorkpadMessage;

#[derive(Debug, Clone)]
pub enum AddSheetMessage {
    PadUpdated(Result<Workpad, UpdateError>),
    SelectKind(SheetKind),
    Name(String),
    Submit,
    Cancel,
}

impl AddSheetMessage {
    pub fn map_to_workpad(self) -> WorkpadMessage {
        match self {
            Self::PadUpdated(result) => WorkpadMessage::PadUpdated(result),
            Self::Cancel => WorkpadMessage::AddSheetCancel,
            m => WorkpadMessage::AddSheetMsg(m),
        }
    }
}

impl std::fmt::Display for AddSheetMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddSheetMessage::")?;
        match self {
            Self::PadUpdated(Ok(workpad)) => write!(f, "PadUpdated({workpad})"),
            Self::PadUpdated(Err(err)) => write!(f, "PadUpdated(ERROR: {err})"),
            Self::SelectKind(k) => write!(f, "SelectKind({k})"),
            Self::Name(n) => write!(f, "Name({n})"),
            Self::Submit => write!(f, "Submit"),
            Self::Cancel => write!(f, "Cancel"),
        }
    }
}

// TODO Focus management
#[derive(Debug)]
pub struct AddSheetUi {
    pad: Workpad,
    existing_names: Vec<String>,
    kind: SheetKind,
    name: String,
    name_error: Option<String>,
}

impl AddSheetUi {
    pub fn new(pad: Workpad) -> Self {
        let existing_names: Vec<String> =
            pad.sheets().map(|sheet| sheet.name().to_owned()).collect();

        let name = (1..)
            .map(|n| format!("Sheet {}", n))
            .find(|n| !existing_names.contains(n))
            .unwrap();
        Self {
            pad,
            existing_names,
            kind: Default::default(),
            name,
            name_error: None,
        }
    }

    pub fn view(&self) -> iced::Element<'_, AddSheetMessage> {
        let mut buttons = button_bar();
        if !self.existing_names.is_empty() {
            buttons = buttons.push(
                dialog_button(t!("Common.Cancel"), style::DialogButtonStyle::Cancel)
                    .on_press(AddSheetMessage::Cancel),
            );
        }
        let mut ok = dialog_button(t!("Common.Ok"), style::DialogButtonStyle::Ok);
        if self.name_error.is_none() {
            ok = ok.on_press(AddSheetMessage::Submit)
        }
        buttons = buttons.push(ok);

        column![column![
            text(t!("AddSheet.Type"))
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
            row![
                kind_button(SheetKind::Worksheet, self.kind == SheetKind::Worksheet),
                kind_button(SheetKind::Textsheet, self.kind == SheetKind::Textsheet)
            ]
            .spacing(SPACE_M)
            .width(Length::Fill),
            vertical_space(SPACE_M),
            horizontal_rule(3),
            vertical_space(SPACE_M),
            text_input(
                t!("SheetName.Label"),
                t!("SheetName.Placeholder"),
                &self.name,
                AddSheetMessage::Name,
                self.name_error.as_ref()
            ),
            buttons
        ]
        .padding(50)
        .align_items(Alignment::Start)]
        .padding(10)
        .spacing(SPACE_S)
        .align_items(Alignment::Start)
        .into()
    }

    pub fn subscription(&self) -> Subscription<AddSheetMessage> {
        if self.existing_names.is_empty() {
            subscription::events_with(|event, _status| {
                handle_ok_key(&event, AddSheetMessage::Submit)
            })
        } else {
            subscription::events_with(|event, _status| {
                handle_ok_and_cancel_keys(&event, AddSheetMessage::Submit, AddSheetMessage::Cancel)
            })
        }
    }

    pub fn update(&mut self, message: AddSheetMessage) -> Command<AddSheetMessage> {
        match message {
            AddSheetMessage::PadUpdated(_) => unreachable!(),
            AddSheetMessage::SelectKind(kind) => {
                debug!(target: "flexpad", %message);
                self.kind = kind;
                Command::none()
            }
            AddSheetMessage::Name(n) => {
                if self.existing_names.contains(&n) {
                    self.name_error = Some(t!("SheetName.AlreadyUsedError"))
                } else if n.is_empty() {
                    self.name_error = Some(t!("SheetName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = n;
                Command::none()
            }
            AddSheetMessage::Submit => {
                debug!(target: "flexpad", %message);
                self.update_pad(WorkpadUpdate::SheetAdd {
                    kind: self.kind,
                    name: self.name.clone(),
                })
            }
            AddSheetMessage::Cancel => unreachable!(),
        }
    }

    pub fn update_pad(&mut self, update: WorkpadUpdate) -> Command<AddSheetMessage> {
        Command::perform(
            super::update_pad(self.pad.master(), update),
            AddSheetMessage::PadUpdated,
        )
    }

    pub fn menu_paths(&self) -> menu::PathVec<AddSheetMessage> {
        menu::PathVec::new()
    }
}

fn kind_button<'a>(kind: SheetKind, selected: bool) -> Element<'a, AddSheetMessage> {
    let txt = match kind {
        SheetKind::Worksheet => t!("SheetKind.Worksheet"),
        SheetKind::Textsheet => t!("SheetKind.Textsheet"),
    };

    let img = match kind {
        SheetKind::Worksheet => images::worksheet(),
        SheetKind::Textsheet => images::textsheet(),
    };

    let style = match selected {
        true => KindButtonContainerStyle::Selected,
        false => KindButtonContainerStyle::NotSelected,
    };

    column![
        container(
            button(image(img).width(48).height(48))
                .on_press(AddSheetMessage::SelectKind(kind))
                .style(theme::Button::Text)
        )
        .style(style),
        text(txt).size(12)
    ]
    .align_items(Alignment::Center)
    .into()
}

enum KindButtonContainerStyle {
    Selected,
    NotSelected,
}

impl From<KindButtonContainerStyle> for theme::Container {
    fn from(value: KindButtonContainerStyle) -> Self {
        theme::Container::Custom(Box::new(value))
    }
}

impl container::StyleSheet for KindButtonContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let palette = style.extended_palette();
        match self {
            KindButtonContainerStyle::Selected => container::Appearance {
                border_radius: 2.0.into(),
                border_width: 1.0,
                border_color: palette.primary.base.color,
                ..Default::default()
            },
            KindButtonContainerStyle::NotSelected => container::Appearance {
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: iced::Color::TRANSPARENT,
                ..Default::default()
            },
        }
    }
}
