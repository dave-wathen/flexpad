use super::util::{FlexpadAction, FLEXPAD_GRID_COLOR};
use crate::ui::workpad_menu;
use flexpad_model::{SheetKind, Workpad, WorkpadMaster, WorkpadUpdate};
use flexpad_toolkit::{button_bar::ButtonBar, menu, prelude::*};
use iced::{
    alignment,
    theme::{self},
    widget::{button, column, container, horizontal_rule, row, text, vertical_space},
    Alignment, Element, Length, Subscription,
};
use rust_i18n::t;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum Message {
    SelectKind(SheetKind),
    Name(String),
    Submit,
    Cancel,
    PadClose,
    PadShowProperties,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddSheetMessage::")?;
        match self {
            Self::SelectKind(k) => write!(f, "SelectKind({k})"),
            Self::Name(n) => write!(f, "Name({n})"),
            Self::Submit => write!(f, "Submit"),
            Self::Cancel => write!(f, "Cancel"),
            Self::PadShowProperties => write!(f, "PadShowProperties"),
            Self::PadClose => write!(f, "PadClose"),
        }
    }
}

pub enum Event {
    None,
    EditPadPropertiesRequested(Workpad),
    CloseWorkpadRequested,
    Cancelled(Workpad),
    Submitted(WorkpadMaster, WorkpadUpdate),
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

    pub fn title(&self) -> String {
        self.pad.name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let mut buttons = ButtonBar::new();
        if !self.existing_names.is_empty() {
            buttons = buttons.push(
                action_button(FlexpadAction::Cancel)
                    .style(style::ButtonStyle::Cancel)
                    .on_press(Message::Cancel),
            );
        }
        let mut ok = action_button(FlexpadAction::Ok).style(style::ButtonStyle::Ok);
        if self.name_error.is_none() {
            ok = ok.on_press(Message::Submit)
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
                Message::Name,
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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::SelectKind(kind) => {
                debug!(target: "flexpad", %message);
                self.kind = kind;
                Event::None
            }
            Message::Name(n) => {
                if self.existing_names.contains(&n) {
                    self.name_error = Some(t!("SheetName.AlreadyUsedError"))
                } else if n.is_empty() {
                    self.name_error = Some(t!("SheetName.EmptyError"))
                } else {
                    self.name_error = None
                }
                self.name = n;
                Event::None
            }
            Message::Cancel => Event::Cancelled(self.pad.clone()),
            Message::Submit => Event::Submitted(
                self.pad.master(),
                WorkpadUpdate::SheetAdd {
                    kind: self.kind,
                    name: self.name.clone(),
                },
            ),
            Message::PadShowProperties => Event::EditPadPropertiesRequested(self.pad.clone()),
            Message::PadClose => Event::CloseWorkpadRequested,
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        menu::PathVec::new()
            .with(workpad_menu::new_blank_workpad(None))
            .with(workpad_menu::new_starter_workpad(None))
            .with(workpad_menu::show_properties(Some(
                Message::PadShowProperties,
            )))
            // TODO No actual delete (since no actual save) at present
            .with(workpad_menu::delete_pad(Some(Message::PadClose)))
            .with(workpad_menu::close_pad(Some(Message::PadClose)))
    }
}

fn kind_button<'a>(kind: SheetKind, selected: bool) -> Element<'a, Message> {
    let action: Action = match kind {
        SheetKind::Worksheet => FlexpadAction::NewWorksheet,
        SheetKind::Textsheet => FlexpadAction::NewTextsheet,
    }
    .into();

    let style = match selected {
        true => KindButtonContainerStyle::Selected,
        false => KindButtonContainerStyle::NotSelected,
    };

    column![
        container(
            button(
                icon(
                    action
                        .icon_codepoint
                        .expect("Add sheet actions must have image codepoints"),
                    ICON_BUTTON_SIZE
                )
                .style(theme::Text::Color(FLEXPAD_GRID_COLOR))
            )
            .on_press(Message::SelectKind(kind))
            .style(theme::Button::Text)
        )
        .style(style),
        text(&action.short_name).size(TEXT_SIZE_LABEL)
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
