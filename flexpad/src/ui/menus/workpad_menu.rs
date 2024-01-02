use crate::ui::util::FlexpadAction;
use flexpad_toolkit::menu;
use rust_i18n::t;

fn root<Message>() -> menu::PathToMenu<Message>
where
    Message: Clone,
{
    menu::root(t!("Menu.Workpad"))
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
    menu::Path::new(root(), FlexpadAction::NewBlank, on_select)
}

pub fn new_starter_workpad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(root(), FlexpadAction::NewStarter, on_select)
}

pub fn show_properties<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(section_1(), FlexpadAction::PadProperties, on_select)
}

pub fn delete_pad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(section_1(), FlexpadAction::PadDelete, on_select)
}

pub fn close_pad<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(section_1(), FlexpadAction::PadClose, on_select)
}
