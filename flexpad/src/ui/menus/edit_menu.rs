use iced::keyboard;
use rust_i18n::t;

use crate::ui::util::{
    key::{command, key, shift},
    menu,
};

fn root<Message>() -> menu::PathToMenu<Message>
where
    Message: Clone,
{
    menu::root(t!("Menus.Edit.Title"))
}

pub fn undo<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        root(),
        t!("Menus.Edit.Undo"),
        Some(command(key(keyboard::KeyCode::Z))),
        on_select,
    )
}

pub fn redo<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        root(),
        t!("Menus.Edit.Redo"),
        Some(command(shift(key(keyboard::KeyCode::Z)))),
        on_select,
    )
}
