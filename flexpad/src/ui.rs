use crate::{
    ui::util::{error::ErrorUi, menu, modal::Modal},
    version::Version,
};
use iced::{window, Application, Command, Settings, Theme};
use rust_i18n::t;
use tracing::debug;

use self::workpad::{WorkpadMessage, WorkpadUI};
use crate::model::workpad::WorkpadMaster;

mod loading;
mod lobby;
mod util;
mod workpad;

pub use util::style;

pub(crate) fn run() -> iced::Result {
    let settings = Settings::default();
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
    Workpad(WorkpadUI),
}

#[derive(Default)]
enum Dialog {
    #[default]
    None,
    Error(util::error::ErrorUi),
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Loading(_) => write!(f, "Loading"),
            Screen::Lobby(_) => write!(f, "FrontScreen"),
            Screen::Workpad(_) => write!(f, "Workpad"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Loading(loading::Message),
    Lobby(lobby::Message),
    WorkpadMsg(WorkpadMessage),
    Error(util::error::Message),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message::")?;
        match self {
            Self::Loading(msg) => msg.fmt(f),
            Self::Lobby(msg) => msg.fmt(f),
            Self::WorkpadMsg(msg) => msg.fmt(f),
            Self::Error(msg) => msg.fmt(f),
        }
    }
}

pub enum Action {
    None,
    StartUi,
    NewBlankWorkpad,
    NewStarterWorkpad,
    CloseDialog,
    #[allow(dead_code)]
    ShowError(String),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action::")?;
        match self {
            Self::None => write!(f, "None"),
            Self::StartUi => write!(f, "StartUi"),
            Self::NewBlankWorkpad => write!(f, "NewBlankWorkpad"),
            Self::NewStarterWorkpad => write!(f, "NewStarterWorkpad"),
            Self::CloseDialog => write!(f, "CloseDialog"),
            Self::ShowError(error) => write!(f, "ShowError({})", error),
        }
    }
}

// TODO Focus management currently missing from iced - not easy to fake up in the meantime

pub struct Flexpad {
    version: Version,
    state: Screen,
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
                state: Screen::Loading(screen),
                dialog: Default::default(),
            },
            Command::batch(vec![
                loading_command.map(Message::Loading),
                window::maximize(true),
            ]),
        )
    }

    fn title(&self) -> String {
        match self.state {
            Screen::Workpad(ref pad) => pad.title(),
            _ => t!("Product"),
        }
    }

    #[tracing::instrument(target = "flexpad", skip_all)]
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        let action = match message.clone() {
            Message::Loading(m) => {
                let Screen::Loading(ui) = &mut self.state else {
                    unreachable!()
                };
                ui.update(m)
            }
            Message::Lobby(m) => {
                let Screen::Lobby(ui) = &mut self.state else {
                    unreachable!()
                };
                ui.update(m)
            }
            Message::Error(m) => {
                let Dialog::Error(ui) = &mut self.dialog else {
                    unreachable!()
                };
                ui.update(m)
            }
            _ => Action::None,
        };

        // TODO Continue making the structure regular

        debug!(target: "flexpad", %action);
        match action {
            Action::StartUi => {
                // Loading has finished
                self.state = Screen::Lobby(lobby::Lobby::new(self.version));
                Command::none()
            }
            Action::NewBlankWorkpad => {
                let workpad = WorkpadMaster::new_blank();
                self.state = Screen::Workpad(WorkpadUI::new(workpad));
                Command::none()
            }
            Action::NewStarterWorkpad => {
                let workpad = WorkpadMaster::new_starter();
                self.state = Screen::Workpad(WorkpadUI::new(workpad));
                Command::none()
            }
            Action::ShowError(error) => {
                self.dialog = Dialog::Error(ErrorUi::new(error));
                Command::none()
            }
            Action::CloseDialog => {
                self.dialog = Dialog::None;
                Command::none()
            }
            Action::None => match self.state {
                Screen::Loading(_) => Command::none(),
                Screen::Lobby(_) => Command::none(),
                Screen::Workpad(ref mut pad) => match message {
                    Message::WorkpadMsg(msg) => match msg {
                        WorkpadMessage::PadClose => {
                            debug!(target: "flexpad", message=%msg);
                            self.state = Screen::Lobby(lobby::Lobby::new(self.version));
                            Command::none()
                        }
                        _ => pad.update(msg).map(Message::WorkpadMsg),
                    },
                    _ => Command::none(),
                },
            },
        }
    }

    #[tracing::instrument(skip_all)]
    fn view(&self) -> iced::Element<'_, Self::Message> {
        debug!(target: "flexpad", state=%self.state, "View");

        let body = match &self.state {
            Screen::Loading(ui) => ui.view().map(Message::Loading),
            Screen::Lobby(ui) => ui.view().map(Message::Lobby),
            Screen::Workpad(pad) => pad.view().map(Message::WorkpadMsg),
        };

        let paths: menu::PathVec<Message> = match &self.state {
            Screen::Loading(ui) => ui.menu_paths().map(Message::Loading),
            Screen::Lobby(ui) => ui.menu_paths().map(Message::Lobby),
            Screen::Workpad(ui) => ui.menu_paths().map(Message::WorkpadMsg),
        };

        let screen = crate::ui::menu::MenuedContent::new(paths, body).into();

        match &self.dialog {
            Dialog::None => screen,
            Dialog::Error(ui) => Modal::new(screen, ui.view().map(Message::Error)).into(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match &self.dialog {
            Dialog::None => match &self.state {
                Screen::Loading(ui) => ui.subscription().map(Message::Loading),
                Screen::Lobby(ui) => ui.subscription().map(Message::Lobby),
                Screen::Workpad(ui) => ui.subscription().map(Message::WorkpadMsg),
            },
            Dialog::Error(ui) => ui.subscription().map(Message::Error),
        }
    }
}
