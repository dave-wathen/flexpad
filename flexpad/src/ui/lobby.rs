use crate::version::Version;

use crate::ui::{
    menu,
    util::{images, SPACE_M, SPACE_S},
    workpad_menu,
};

use iced::{
    alignment, theme,
    widget::{button, column, horizontal_rule, image, row, text},
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
        let image_button = |img, title, msg| {
            column![
                button(image(img).width(48).height(48))
                    .on_press(msg)
                    .style(theme::Button::Text),
                text(title).size(12)
            ]
            .align_items(Alignment::Center)
        };

        column![
            image(images::app()).width(200).height(200),
            text(self.version.description()).size(12),
            horizontal_rule(3),
            text(t!("Workpads.Create"))
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
            row![
                image_button(
                    images::workpad_no_sheets(),
                    t!("Workpads.Blank"),
                    Message::NewBlankWorkpad
                ),
                image_button(
                    images::workpad_and_sheets(),
                    t!("Workpads.Starter"),
                    Message::NewStarterWorkpad
                )
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
