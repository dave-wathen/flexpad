use iced::{
    alignment,
    widget::{container, text},
    Command, Length,
};
use rust_i18n::t;

use super::{menu, style, Action};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<(), String>),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loading::Message::")?;
        match self {
            Self::Loaded(result) => write!(f, "Loaded({result:?})"),
        }
    }
}

pub struct Loading;

impl Loading {
    pub fn new() -> (Self, Command<Message>) {
        (Self, iced::Command::perform(load(), Message::Loaded))
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self) -> iced::Element<'a, Message> {
        container(
            text(t!("Common.Loading"))
                .style(style::TextStyle::Loading)
                .horizontal_alignment(alignment::Horizontal::Center)
                .size(40),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y()
        .center_x()
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Loaded(_) => Action::StartUi,
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        menu::PathVec::new()
    }
}

pub async fn load() -> Result<(), String> {
    // TODO Loading activities
    Ok(())
}
