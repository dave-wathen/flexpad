use iced::{
    alignment,
    widget::{container, text},
    Length,
};
use rust_i18n::t;

use super::{menu, style, Message};

pub(super) fn view<'a>() -> iced::Element<'a, Message> {
    container(
        text(t!("Common.Loading"))
            .style(style::TextStyle::Default)
            .horizontal_alignment(alignment::Horizontal::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .center_x()
    .into()
}

pub(crate) fn menu_paths() -> Vec<menu::Path<Message>> {
    vec![]
}
