use flexpad_toolkit::menu;
use rust_i18n::t;

use crate::ui::util::FlexpadAction;

fn root<Message>() -> menu::PathToMenu<Message>
where
    Message: Clone,
{
    menu::root(t!("Menu.Edit"))
}

pub fn undo<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(root(), FlexpadAction::Undo, on_select)
}

pub fn redo<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(root(), FlexpadAction::Redo, on_select)
}
