use rust_i18n::t;

use crate::ui::util::{ACTION_REDO, ACTION_UNDO};
use flexpad_toolkit::menu;

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
    menu::Path::new(root(), &ACTION_UNDO, on_select)
}

pub fn redo<Message>(on_select: Option<Message>) -> menu::Path<Message>
where
    Message: Clone,
{
    menu::Path::new(root(), &ACTION_REDO, on_select)
}
