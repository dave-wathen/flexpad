use iced::widget::image::Handle;

pub fn app() -> Handle {
    Handle::from_memory(include_bytes!("./images/flexpad.png"))
}

#[allow(dead_code)]
pub fn cancel() -> Handle {
    Handle::from_memory(include_bytes!("./images/cancel.png"))
}

#[allow(dead_code)]
pub fn close() -> Handle {
    Handle::from_memory(include_bytes!("./images/close.png"))
}

pub fn delete() -> Handle {
    Handle::from_memory(include_bytes!("./images/delete.png"))
}

pub fn edit() -> Handle {
    Handle::from_memory(include_bytes!("./images/edit.png"))
}

pub fn expand_more() -> Handle {
    Handle::from_memory(include_bytes!("./images/expand_more.png"))
}

pub fn fx() -> Handle {
    Handle::from_memory(include_bytes!("./images/fx.png"))
}

#[allow(dead_code)]
pub fn more_horiz() -> Handle {
    Handle::from_memory(include_bytes!("./images/more_horiz.png"))
}

#[allow(dead_code)]
pub fn more_vert() -> Handle {
    Handle::from_memory(include_bytes!("./images/more_vert.png"))
}

pub fn print() -> Handle {
    Handle::from_memory(include_bytes!("./images/print.png"))
}

pub fn redo() -> Handle {
    Handle::from_memory(include_bytes!("./images/redo.png"))
}

pub fn settings() -> Handle {
    Handle::from_memory(include_bytes!("./images/settings.png"))
}

pub fn workpad() -> Handle {
    Handle::from_memory(include_bytes!("./images/workpad.png"))
}

pub fn undo() -> Handle {
    Handle::from_memory(include_bytes!("./images/undo.png"))
}
