use iced::{
    alignment,
    widget::{container, text},
    Command, Length,
};
use rust_i18n::t;

use super::style;
use crate::ui;

#[derive(Debug, Clone)]

pub struct Loading;

impl Loading {
    pub fn new() -> (Self, Command<ui::Message>) {
        (
            Self,
            iced::Command::perform(load(), ui::Message::LoadingComplete),
        )
    }

    pub fn view<'a>(&self) -> iced::Element<'a, ui::Message> {
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
