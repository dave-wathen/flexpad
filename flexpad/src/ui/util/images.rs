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

#[allow(dead_code)]
pub fn delete() -> Handle {
    Handle::from_memory(include_bytes!("./images/delete.png"))
}

#[allow(dead_code)]
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

pub fn workpad_no_sheets() -> Handle {
    Handle::from_memory(include_bytes!("./images/workpad_no_sheets.png"))
}

pub fn workpad_and_sheets() -> Handle {
    Handle::from_memory(include_bytes!("./images/workpad_and_sheets.png"))
}

pub fn worksheet() -> Handle {
    Handle::from_memory(include_bytes!("./images/worksheet.png"))
}

pub fn textsheet() -> Handle {
    Handle::from_memory(include_bytes!("./images/textsheet.png"))
}
