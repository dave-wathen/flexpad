use crate::{menu::workpad_menu, version::Version, FlexpadAction, FLEXPAD_GRID_COLOR};
use flexpad_toolkit::{menu, prelude::*};
use iced::{
    alignment, theme,
    widget::{self, button, column, horizontal_rule, image::Handle, row, text},
    Alignment, Length,
};
use rust_i18n::t;

#[derive(Debug, Clone)]
pub enum Message {
    NewBlankWorkpad,
    NewStarterWorkpad,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lobby::Message::")?;
        match self {
            Self::NewBlankWorkpad => write!(f, "NewBlankWorkpad"),
            Self::NewStarterWorkpad => write!(f, "NewStarterWorkpad"),
        }
    }
}

pub enum Event {
    NewBlankWorkpadRequested,
    NewStarterWorkpadRequested,
}

pub struct Lobby {
    version: Version,
}

impl Lobby {
    pub fn new(version: Version) -> Self {
        Self { version }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self) -> iced::Element<'a, Message> {
        let app_image = Handle::from_memory(include_bytes!("../../resources/flexpad.png"));

        fn image_button<'a>(action: impl Into<Action>, msg: Message) -> iced::Element<'a, Message> {
            let action = action.into();
            let codepoint = action
                .icon_codepoint
                .expect("Lobby actions must have a codepoint");
            column![
                tooltip(
                    &action,
                    button(
                        icon(codepoint, ICON_BUTTON_SIZE)
                            .style(theme::Text::Color(FLEXPAD_GRID_COLOR))
                    )
                    .on_press(msg)
                    .style(theme::Button::Text),
                    TooltipPosition::FollowCursor
                ),
                label(&action.short_name)
            ]
            .align_items(Alignment::Center)
            .into()
        }

        column![
            widget::image(app_image).width(200).height(200),
            label(self.version.description()),
            horizontal_rule(3),
            text(t!("Workpads.Create"))
                .size(TEXT_SIZE_APP_TITLE)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
            row![
                image_button(FlexpadAction::NewBlank, Message::NewBlankWorkpad),
                image_button(FlexpadAction::NewStarter, Message::NewStarterWorkpad)
            ]
            .spacing(SPACE_M)
            .width(Length::Fill),
            horizontal_rule(3),
            text(t!("Workpads.Reopen"))
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
        ]
        .width(Length::Fill)
        .spacing(SPACE_S)
        .padding(50)
        .align_items(Alignment::Center)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::NewBlankWorkpad => Event::NewBlankWorkpadRequested,
            Message::NewStarterWorkpad => Event::NewStarterWorkpadRequested,
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        menu::PathVec::new()
            .with(workpad_menu::new_blank_workpad(Some(
                Message::NewBlankWorkpad,
            )))
            .with(workpad_menu::new_starter_workpad(Some(
                Message::NewStarterWorkpad,
            )))
    }
}
