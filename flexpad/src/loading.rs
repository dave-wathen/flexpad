use flexpad_toolkit::prelude::*;
use iced::{
    alignment,
    widget::{container, text},
    Command, Length,
};
use rust_i18n::t;

#[derive(Debug, Clone)]

pub struct Loading;

impl Loading {
    pub fn new() -> (Self, Command<super::Message>) {
        (
            Self,
            iced::Command::perform(load(), super::Message::LoadingComplete),
        )
    }

    pub fn view<'a>(&self) -> iced::Element<'a, super::Message> {
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
}

pub async fn load() -> Result<(), String> {
    // TODO Loading activities
    Ok(())
}
