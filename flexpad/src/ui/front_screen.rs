use crate::version::Version;

use super::{
    images,
    key::{command, key, shift},
    menu, Message, SPACE_M, SPACE_S,
};

use iced::{
    alignment, keyboard, theme,
    widget::{button, column, horizontal_rule, image, row, text},
    Alignment, Length,
};
use rust_i18n::t;

pub(super) fn view<'a>(version: &Version) -> iced::Element<'a, Message> {
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
        text(version.description()).size(12),
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

pub(super) fn menu_paths() -> Vec<menu::Path<Message>> {
    let workpad = menu::root(t!("Menus.Workpad.Title"));
    vec![
        workpad.item(
            menu::item(t!("Menus.Workpad.NewBlank"))
                .shortcut(command(key(keyboard::KeyCode::N)))
                .on_select(Message::NewBlankWorkpad),
        ),
        workpad.item(
            menu::item(t!("Menus.Workpad.NewStarter"))
                .shortcut(shift(command(key(keyboard::KeyCode::N))))
                .on_select(Message::NewStarterWorkpad),
        ),
    ]
}
