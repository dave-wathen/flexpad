use crate::{
    model::workpad::{Sheet, Workpad, WorkpadMaster, WorkpadUpdate},
    ui::{
        menu,
        util::key::{command, key, shift},
    },
};
use iced::{keyboard, Command, Subscription};
use rust_i18n::t;
use tracing::debug;

use crate::ui;

mod active_cell;
mod active_sheet;
mod add_sheet;
mod inactive_cell;

#[derive(Debug)]
enum Screen {
    ActiveSheet(active_sheet::ActiveSheetUi),
    AddSheet(add_sheet::AddSheetUi),
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Screen::")?;
        match self {
            Screen::ActiveSheet(_) => write!(f, "ActiveSheet"),
            Screen::AddSheet(_) => write!(f, "AddSheet"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Sub views
    ActiveSheet(active_sheet::Message),
    AddSheet(add_sheet::Message),
    // Pad actions
    PadClose,
    ShowProperties,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkpadMessage::")?;
        match self {
            Self::ActiveSheet(msg) => msg.fmt(f),
            Self::AddSheet(msg) => msg.fmt(f),
            Self::PadClose => write!(f, "PadClose"),
            Self::ShowProperties => write!(f, "PadShowProperties"),
        }
    }
}

pub enum Event {
    None,
    EditPadPropertiesRequested(Workpad),
    EditSheetPropertiesRequested(Sheet),
    UpdateRequested(WorkpadMaster, WorkpadUpdate),
    CloseWorkpadRequested,
}

#[derive(Debug)]
pub struct WorkpadUi {
    pad: Workpad,
    screen: Screen,
}

impl WorkpadUi {
    pub fn new(pad_master: WorkpadMaster) -> Self {
        let pad = pad_master.active_version();
        let screen = match pad.active_sheet() {
            Some(sheet) => Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet)),
            None => Screen::AddSheet(add_sheet::AddSheetUi::new(pad.clone())),
        };

        Self { pad, screen }
    }

    pub fn title(&self) -> String {
        self.pad.name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        debug!(target: "flexpad", state=%self.screen, "Workpad View");

        match self.screen {
            Screen::ActiveSheet(ref child_ui) => child_ui.view().map(Message::ActiveSheet),
            Screen::AddSheet(ref child_ui) => child_ui.view().map(Message::AddSheet),
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        let workpad_menu = menu::root(t!("Menus.Workpad.Title"));
        menu::PathVec::new()
            .with(
                workpad_menu.item(
                    menu::item(t!("Menus.Workpad.NewBlank"))
                        .shortcut(command(key(keyboard::KeyCode::N))),
                ),
            )
            .with(
                workpad_menu.item(
                    menu::item(t!("Menus.Workpad.NewStarter"))
                        .shortcut(shift(command(key(keyboard::KeyCode::N)))),
                ),
            )
            .with(
                workpad_menu.section("1").item(
                    menu::item(t!("Menus.Workpad.PadShowProperties"))
                        .shortcut(command(key(keyboard::KeyCode::Comma)))
                        .on_select(Message::ShowProperties),
                ),
            )
            .with(
                // TODO No actual delete (since no actual save) at present
                workpad_menu.section("1").item(
                    menu::item(t!("Menus.Workpad.PadDelete"))
                        .shortcut(command(key(keyboard::KeyCode::Delete)))
                        .on_select(Message::PadClose),
                ),
            )
            .with(
                workpad_menu.section("1").item(
                    menu::item(t!("Menus.Workpad.PadClose"))
                        .shortcut(command(key(keyboard::KeyCode::W)))
                        .on_select(Message::PadClose),
                ),
            )
            .extend(match &self.screen {
                Screen::ActiveSheet(ui) => ui.menu_paths().map(Message::ActiveSheet),
                Screen::AddSheet(ui) => ui.menu_paths().map(Message::AddSheet),
            })
    }

    pub(crate) fn subscription(&self) -> iced::Subscription<Message> {
        match self.screen {
            Screen::ActiveSheet(_) => Subscription::none(),
            Screen::AddSheet(ref ui) => ui.subscription().map(Message::AddSheet),
        }
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            // Sub views
            Message::ActiveSheet(msg) => {
                let Screen::ActiveSheet(ui) = &mut self.screen else {
                    unreachable!()
                };
                match ui.update(msg) {
                    active_sheet::Event::None => Event::None,
                    active_sheet::Event::EditSheetPropertiesRequested(sheet) => {
                        Event::EditSheetPropertiesRequested(sheet)
                    }
                    active_sheet::Event::AddSheetRequested => {
                        self.screen =
                            Screen::AddSheet(add_sheet::AddSheetUi::new(self.pad.clone()));
                        Event::None
                    }
                    active_sheet::Event::UpdateRequested(master, update) => {
                        Event::UpdateRequested(master, update)
                    }
                }
            }
            Message::AddSheet(msg) => {
                let Screen::AddSheet(ui) = &mut self.screen else {
                    unreachable!()
                };
                match ui.update(msg) {
                    add_sheet::Event::None => Event::None,
                    add_sheet::Event::Cancelled => {
                        // Can only cancel if there are sheets present
                        let sheet = self.pad.active_sheet().unwrap();
                        self.screen = Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet));
                        Event::None
                    }
                    add_sheet::Event::Submitted(master, update) => {
                        Event::UpdateRequested(master, update)
                    }
                }
            }
            // Pad actions
            Message::PadClose => Event::CloseWorkpadRequested,
            Message::ShowProperties => Event::EditPadPropertiesRequested(self.pad.clone()),
        }
    }

    pub fn pad_updated(&mut self, pad: Workpad) -> Command<ui::Message> {
        self.pad = pad.clone();
        self.screen = match pad.active_sheet() {
            Some(sheet) => Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet)),
            None => Screen::AddSheet(add_sheet::AddSheetUi::new(pad)),
        };
        match &self.screen {
            Screen::ActiveSheet(_) => active_sheet::get_viewport()
                .map(Message::ActiveSheet)
                .map(ui::Message::Workpad),
            Screen::AddSheet(_) => Command::none(),
        }
    }
}
