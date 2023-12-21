use crate::{
    model::workpad::{UpdateResult, WorkpadUpdate},
    ui::util::{
        error,
        menu::{self, PathVec},
        modal::Modal,
    },
    version::Version,
};
use iced::{window, Application, Command, Settings, Theme};
use rust_i18n::t;
use tracing::{debug, info};

use crate::model::workpad::WorkpadMaster;

mod active_sheet;
mod add_sheet;
mod loading;
mod lobby;
mod pad_properties;
mod sheet_properties;
mod util;
pub mod widget {
    pub mod active_cell;
    pub mod inactive_cell;
}
mod menus {
    pub mod edit_menu;
    pub mod workpad_menu;
}
pub use menus::edit_menu;
pub use menus::workpad_menu;

pub use util::style;

pub(crate) fn run() -> iced::Result {
    let settings = Settings {
        fonts: vec![include_bytes!("../fonts/flexpad-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    };
    Flexpad::run(settings)
}

trait Scrn<Message>
where
    Message: Clone,
{
    fn subscription(&self) -> iced::Subscription<Message>;

    fn view<'a>(&self) -> iced::Element<'a, Message>;

    fn menu_paths(&self) -> Vec<menu::Path<Message>>;
}

enum Screen {
    Loading(loading::Loading),
    Lobby(lobby::Lobby),
    ActiveSheet(active_sheet::ActiveSheetUi),
    AddSheet(add_sheet::AddSheetUi),
}

#[derive(Default)]
enum Dialog {
    #[default]
    None,
    Error(util::error::ErrorUi),
    PadProperties(pad_properties::PadPropertiesUi),
    SheetProperties(sheet_properties::SheetPropertiesUi),
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Loading(_) => write!(f, "Loading"),
            Screen::Lobby(_) => write!(f, "FrontScreen"),
            Screen::ActiveSheet(_) => write!(f, "ActiveSheet"),
            Screen::AddSheet(_) => write!(f, "AddSheet"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataEvent {
    PadOpened(WorkpadMaster),
    PadUpdated(UpdateResult),
}

impl std::fmt::Display for DataEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataEvent::")?;
        match self {
            Self::PadOpened(master) => write!(
                f,
                "Opened id:{} version:({}, {})",
                master.id(),
                master.active_version().version().0,
                master.active_version().version().1
            ),
            Self::PadUpdated(Ok(pad)) => write!(
                f,
                "Message::PadUpdated(Ok) id:{} version:({}, {})",
                pad.id(),
                pad.version().0,
                pad.version().1,
            ),
            Self::PadUpdated(Err(error)) => {
                write!(f, "Message::PadUpdated(ERROR) {}", error)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadingComplete(Result<(), String>),
    Lobby(lobby::Message),
    ActiveSheet(active_sheet::Message),
    AddSheet(add_sheet::Message),
    Error(util::error::Message),
    SheetProperties(sheet_properties::Message),
    PadProperties(pad_properties::Message),
    Data(DataEvent),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadingComplete(Ok(_)) => write!(f, "Message::LoadingComplete(OK)"),
            Self::LoadingComplete(Err(error)) => {
                write!(f, "Message::LoadingComplete(ERROR) {}", error)
            }
            Self::Lobby(msg) => msg.fmt(f),
            Self::ActiveSheet(msg) => msg.fmt(f),
            Self::AddSheet(msg) => msg.fmt(f),
            Self::Error(msg) => msg.fmt(f),
            Self::PadProperties(msg) => msg.fmt(f),
            Self::SheetProperties(msg) => msg.fmt(f),
            Self::Data(msg) => msg.fmt(f),
        }
    }
}

// TODO Focus management currently missing from iced - not easy to fake up in the meantime

pub struct Flexpad {
    version: Version,
    screen: Screen,
    dialog: Dialog,
}

impl Application for Flexpad {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Flexpad, Command<Message>) {
        let (screen, loading_command) = loading::Loading::new();
        (
            Self {
                version: Default::default(),
                screen: Screen::Loading(screen),
                dialog: Default::default(),
            },
            Command::batch(vec![
                loading_command,
                window::maximize(window::Id::MAIN, true),
            ]),
        )
    }

    fn title(&self) -> String {
        match &self.screen {
            Screen::ActiveSheet(ui) => ui.title(),
            Screen::AddSheet(ui) => ui.title(),
            _ => t!("Product"),
        }
    }

    #[tracing::instrument(target = "flexpad", skip_all)]
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::LoadingComplete(Err(error)) => panic!("{}", error),
            Message::LoadingComplete(Ok(_)) => {
                self.screen = Screen::Lobby(lobby::Lobby::new(self.version));
                Command::none()
            }
            Message::Lobby(m) => {
                let Screen::Lobby(ui) = &mut self.screen else {
                    unreachable!()
                };
                match ui.update(m) {
                    lobby::Event::NewBlankWorkpadRequested => new_blank_workpad(),
                    lobby::Event::NewStarterWorkpadRequested => new_starter_workpad(),
                }
            }
            Message::ActiveSheet(m) => {
                let Screen::ActiveSheet(ui) = &mut self.screen else {
                    unreachable!()
                };
                match ui.update(m) {
                    active_sheet::Event::None => Command::none(),
                    active_sheet::Event::EditPadPropertiesRequested(pad) => {
                        self.dialog =
                            Dialog::PadProperties(pad_properties::PadPropertiesUi::new(pad));
                        Command::none()
                    }
                    active_sheet::Event::CloseWorkpadRequested => {
                        self.screen = Screen::Lobby(lobby::Lobby::new(self.version));
                        Command::none()
                    }
                    active_sheet::Event::EditSheetPropertiesRequested(sheet) => {
                        self.dialog = Dialog::SheetProperties(
                            sheet_properties::SheetPropertiesUi::new(sheet),
                        );
                        Command::none()
                    }
                    active_sheet::Event::AddSheetRequested(pad) => {
                        self.screen = Screen::AddSheet(add_sheet::AddSheetUi::new(pad));
                        Command::none()
                    }
                    active_sheet::Event::UpdateRequested(master, update) => {
                        update_pad(master, update)
                    }
                }
            }
            Message::AddSheet(m) => {
                let Screen::AddSheet(ui) = &mut self.screen else {
                    unreachable!()
                };
                match ui.update(m) {
                    add_sheet::Event::None => Command::none(),
                    add_sheet::Event::EditPadPropertiesRequested(pad) => {
                        self.dialog =
                            Dialog::PadProperties(pad_properties::PadPropertiesUi::new(pad));
                        Command::none()
                    }
                    add_sheet::Event::CloseWorkpadRequested => {
                        self.screen = Screen::Lobby(lobby::Lobby::new(self.version));
                        Command::none()
                    }
                    add_sheet::Event::Cancelled(pad) => {
                        // Can only cancel if there are sheets present
                        let sheet = pad.active_sheet().unwrap();
                        self.screen = Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet));
                        Command::none()
                    }
                    add_sheet::Event::Submitted(master, update) => update_pad(master, update),
                }
            }
            Message::Error(m) => {
                let Dialog::Error(ui) = &mut self.dialog else {
                    unreachable!()
                };
                match ui.update(m) {
                    error::Event::Acknowledged => {
                        self.dialog = Dialog::None;
                        Command::none()
                    }
                }
            }
            Message::PadProperties(m) => {
                let Dialog::PadProperties(ui) = &mut self.dialog else {
                    unreachable!()
                };
                match ui.update(m) {
                    pad_properties::Event::None => Command::none(),
                    pad_properties::Event::Cancelled => {
                        self.dialog = Dialog::None;
                        Command::none()
                    }
                    pad_properties::Event::Submitted(master, update) => {
                        self.dialog = Dialog::None;
                        update_pad(master, update)
                    }
                }
            }
            Message::SheetProperties(m) => {
                let Dialog::SheetProperties(ui) = &mut self.dialog else {
                    unreachable!()
                };
                match ui.update(m) {
                    sheet_properties::Event::None => Command::none(),
                    sheet_properties::Event::Cancelled => {
                        self.dialog = Dialog::None;
                        Command::none()
                    }
                    sheet_properties::Event::Submitted(master, update) => {
                        self.dialog = Dialog::None;
                        update_pad(master, update)
                    }
                }
            }
            Message::Data(event) => match event {
                DataEvent::PadOpened(master) => {
                    let pad = master.active_version();
                    self.screen = match pad.active_sheet() {
                        Some(sheet) => Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet)),
                        None => Screen::AddSheet(add_sheet::AddSheetUi::new(pad.clone())),
                    };
                    Command::none()
                }
                DataEvent::PadUpdated(Ok(pad)) => match pad.active_sheet() {
                    Some(sheet) => {
                        if let Screen::ActiveSheet(ui) = &mut self.screen {
                            ui.pad_updated(pad).map(Message::ActiveSheet)
                        } else {
                            self.screen =
                                Screen::ActiveSheet(active_sheet::ActiveSheetUi::new(sheet));
                            Command::none()
                        }
                    }
                    None => {
                        self.screen = Screen::AddSheet(add_sheet::AddSheetUi::new(pad));
                        Command::none()
                    }
                },
                DataEvent::PadUpdated(Err(err)) => {
                    self.dialog = Dialog::Error(error::ErrorUi::new(err.to_string()));
                    Command::none()
                }
            },
        }
    }

    #[tracing::instrument(skip_all)]
    fn view(&self) -> iced::Element<'_, Self::Message> {
        debug!(target: "flexpad", state=%self.screen, "View");

        let body = match &self.screen {
            Screen::Loading(ui) => ui.view(),
            Screen::Lobby(ui) => ui.view().map(Message::Lobby),
            Screen::ActiveSheet(ui) => ui.view().map(Message::ActiveSheet),
            Screen::AddSheet(ui) => ui.view().map(Message::AddSheet),
        };

        let paths: menu::PathVec<Message> = match &self.screen {
            Screen::Loading(_) => PathVec::new(),
            Screen::Lobby(ui) => ui.menu_paths().map(Message::Lobby),
            Screen::ActiveSheet(ui) => ui.menu_paths().map(Message::ActiveSheet),
            Screen::AddSheet(ui) => ui.menu_paths().map(Message::AddSheet),
        };

        let screen = crate::ui::menu::MenuedContent::new(paths, body).into();

        match &self.dialog {
            Dialog::None => screen,
            Dialog::Error(ui) => Modal::new(screen, ui.view().map(Message::Error)).into(),
            Dialog::PadProperties(ui) => {
                Modal::new(screen, ui.view().map(Message::PadProperties)).into()
            }
            Dialog::SheetProperties(ui) => {
                Modal::new(screen, ui.view().map(Message::SheetProperties)).into()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match &self.dialog {
            Dialog::None => match &self.screen {
                Screen::Loading(_) => iced::Subscription::none(),
                Screen::Lobby(ui) => ui.subscription().map(Message::Lobby),
                Screen::ActiveSheet(ui) => ui.subscription().map(Message::ActiveSheet),
                Screen::AddSheet(ui) => ui.subscription().map(Message::AddSheet),
            },
            Dialog::Error(ui) => ui.subscription().map(Message::Error),
            Dialog::PadProperties(ui) => ui.subscription().map(Message::PadProperties),
            Dialog::SheetProperties(ui) => ui.subscription().map(Message::SheetProperties),
        }
    }
}

/// Create a [`Command`] to open a new blank workpad
fn new_blank_workpad() -> Command<Message> {
    Command::perform(new_blank_workpad_async(), DataEvent::PadOpened).map(Message::Data)
}

/// Create a [`Command`] to open a new starter workpad
fn new_starter_workpad() -> Command<Message> {
    Command::perform(new_starter_workpad_async(), DataEvent::PadOpened).map(Message::Data)
}

/// Create a [`Command`] to update a workpad
fn update_pad(master: WorkpadMaster, update: WorkpadUpdate) -> Command<Message> {
    Command::perform(update_pad_async(master, update), DataEvent::PadUpdated).map(Message::Data)
}

async fn new_blank_workpad_async() -> WorkpadMaster {
    info!(target: "flexpad", "new_blank_workpad");
    WorkpadMaster::new_blank()
}

async fn new_starter_workpad_async() -> WorkpadMaster {
    info!(target: "flexpad", "new_starter_workpad");
    WorkpadMaster::new_starter()
}

async fn update_pad_async(mut master: WorkpadMaster, update: WorkpadUpdate) -> UpdateResult {
    info!(target: "flexpad", %update, "Model update");
    master.update(update)
}
