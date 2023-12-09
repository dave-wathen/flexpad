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
    menu::root(t!("Menus.Workpad.Title"))
}

fn section_1<Message>() -> menu::PathToMenuSection<Message>
where
    Message: Clone,
{
    root().section("1")
}

pub fn new_blank_workpad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        root(),
        t!("Menus.Workpad.NewBlank"),
        Some(command(key(keyboard::KeyCode::N))),
        on_select,
    )
}

pub fn new_starter_workpad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        root(),
        t!("Menus.Workpad.NewStarter"),
        Some(command(shift(key(keyboard::KeyCode::N)))),
        on_select,
    )
}

pub fn show_properties<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        section_1(),
        t!("Menus.Workpad.PadShowProperties"),
        Some(command(key(keyboard::KeyCode::Comma))),
        on_select,
    )
}

pub fn delete_pad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        section_1(),
        t!("Menus.Workpad.PadDelete"),
        Some(command(key(keyboard::KeyCode::Delete))),
        on_select,
    )
}

pub fn close_pad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(
        section_1(),
        t!("Menus.Workpad.PadClose"),
        Some(command(key(keyboard::KeyCode::W))),
        on_select,
    )
}
