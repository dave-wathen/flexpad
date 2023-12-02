use std::collections::HashMap;

use crate::{
    display_iter,
    model::workpad::{SheetId, UpdateError, UpdateResult, Workpad, WorkpadMaster, WorkpadUpdate},
    ui::{
        menu,
        util::{
            key::{command, key, shift},
            modal::Modal,
        },
    },
};
use flexpad_grid::{scroll::get_viewport, Viewport};
use iced::{keyboard, widget::button, Color, Command, Subscription};
use rust_i18n::t;
use tracing::{debug, error, info};

use self::{
    active_sheet::{ActiveSheetMessage, ActiveSheetUi},
    add_sheet::{AddSheetMessage, AddSheetUi},
    pad_properties::{PadPropertiesMessage, PadPropertiesUi},
    sheet_properties::{SheetPropertiesMessage, SheetPropertiesUi},
};

use super::util::error::ErrorUi;

mod active_cell;
mod active_sheet;
mod add_sheet;
mod inactive_cell;
mod pad_properties;
mod sheet_properties;

#[derive(Debug)]
enum State {
    ActiveSheet(ActiveSheetUi),
    AddSheet(AddSheetUi),
}

impl State {
    // fn new(pad: &Workpad) -> Self {
    //     match pad.active_sheet() {
    //         Some(active_sheet) => State::ActiveSheet(ActiveSheetUi::new(active_sheet)),
    //         None => State::AddSheet(AddSheetUi::new(pad.clone())),
    //     }
    // }

    fn update_active_sheet(&mut self, msg: ActiveSheetMessage) -> Command<WorkpadMessage> {
        match self {
            State::ActiveSheet(ref mut ui) => {
                ui.update(msg).map(ActiveSheetMessage::map_to_workpad)
            }
            _ => unreachable!(),
        }
    }

    fn update_add_sheet(&mut self, msg: AddSheetMessage) -> Command<WorkpadMessage> {
        match self {
            State::AddSheet(ref mut ui) => ui.update(msg).map(AddSheetMessage::map_to_workpad),
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::ActiveSheet(_) => write!(f, "ActiveSheet"),
            State::AddSheet(_) => write!(f, "AddSheet"),
        }
    }
}

#[derive(Debug, Default)]
enum ShowModal {
    #[default]
    None,
    ShowError(ErrorUi),
    PadProperties(PadPropertiesUi),
    SheetProperties(SheetPropertiesUi),
}

impl ShowModal {
    fn update_pad_properties(&mut self, msg: PadPropertiesMessage) -> Command<WorkpadMessage> {
        match self {
            Self::PadProperties(ref mut modal) => {
                modal.update(msg).map(PadPropertiesMessage::map_to_workpad)
            }
            _ => unreachable!(),
        }
    }

    fn update_sheet_properties(&mut self, msg: SheetPropertiesMessage) -> Command<WorkpadMessage> {
        match self {
            Self::SheetProperties(ref mut modal) => modal
                .update(msg)
                .map(SheetPropertiesMessage::map_to_workpad),
            _ => unreachable!(),
        }
    }

    fn into_update(self) -> WorkpadUpdate {
        match self {
            Self::SheetProperties(props) => props.into_update(),
            Self::PadProperties(props) => props.into_update(),
            Self::ShowError(_) => unreachable!(),
            Self::None => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkpadMessage {
    NoOp,             // TODO Temporary
    OpenMenu(String), // TODO Temporary until system menus
    Multi(Vec<WorkpadMessage>),
    // Modal screens
    ModalSubmit,
    ModalCancel,
    PadPropertiesMsg(PadPropertiesMessage),
    SheetPropertiesMsg(SheetPropertiesMessage),
    // Sub views
    ActiveSheetMsg(ActiveSheetMessage),
    AddSheetMsg(AddSheetMessage),
    AddSheetCancel,
    // Pad actions
    PadUpdated(Result<Workpad, UpdateError>),
    PadClose,
    PadShowProperties,
    PadAddSheet,
    SheetDelete(SheetId),
    SheetShowProperties(SheetId),
}

impl std::fmt::Display for WorkpadMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkpadMessage::")?;
        match self {
            Self::NoOp => write!(f, "NoOp"),
            Self::OpenMenu(name) => write!(f, "OpenMenu({name})"),
            Self::Multi(msgs) => {
                f.write_str("Multi(")?;
                display_iter(msgs.iter(), f)?;
                f.write_str(")")
            }
            Self::ModalSubmit => write!(f, "ModalSubmit"),
            Self::ModalCancel => write!(f, "ModalCancel"),
            Self::PadPropertiesMsg(msg) => msg.fmt(f),
            Self::SheetPropertiesMsg(msg) => msg.fmt(f),
            Self::ActiveSheetMsg(msg) => msg.fmt(f),
            Self::AddSheetMsg(msg) => msg.fmt(f),
            Self::AddSheetCancel => write!(f, "AddSheetCancel"),
            Self::PadUpdated(Ok(workpad)) => write!(f, "PadUpdated({workpad})"),
            Self::PadUpdated(Err(err)) => write!(f, "PadUpdated(ERROR: {err})"),
            Self::PadClose => write!(f, "PadClose"),
            Self::PadShowProperties => write!(f, "PadShowProperties"),
            Self::PadAddSheet => write!(f, "PadAddSheet"),
            Self::SheetDelete(id) => write!(f, "SheetDelete({id})"),
            Self::SheetShowProperties(id) => write!(f, "SheetShowProperties({id})"),
        }
    }
}

#[derive(Debug)]
pub struct WorkpadUI {
    pad_master: WorkpadMaster,
    pad: Workpad,
    state: State,
    modal: ShowModal,
    sheet_viewports: HashMap<SheetId, Viewport>,
}

impl WorkpadUI {
    pub fn new(pad_master: WorkpadMaster) -> Self {
        let pad = pad_master.active_version();
        let state = match pad.active_sheet() {
            Some(sheet) => State::ActiveSheet(ActiveSheetUi::new(sheet, None)),
            None => State::AddSheet(AddSheetUi::new(pad.clone())),
        };

        Self {
            pad_master,
            pad,
            state,
            modal: Default::default(),
            sheet_viewports: Default::default(),
        }
    }

    pub fn title(&self) -> String {
        self.pad.name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, WorkpadMessage> {
        debug!(target: "flexpad", state=%self.state, "Workpad View");

        let screen = match self.state {
            State::ActiveSheet(ref child_ui) => {
                child_ui.view().map(ActiveSheetMessage::map_to_workpad)
            }
            State::AddSheet(ref child_ui) => child_ui.view().map(AddSheetMessage::map_to_workpad),
        };

        match &self.modal {
            ShowModal::None => screen,
            ShowModal::PadProperties(ui) => {
                Modal::new(screen, ui.view().map(PadPropertiesMessage::map_to_workpad)).into()
            }
            ShowModal::SheetProperties(ui) => Modal::new(
                screen,
                ui.view().map(SheetPropertiesMessage::map_to_workpad),
            )
            .into(),
            ShowModal::ShowError(ui) => {
                Modal::new(screen, ui.view().map(|_| WorkpadMessage::ModalCancel)).into()
            }
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<WorkpadMessage> {
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
                        .on_select(WorkpadMessage::PadShowProperties),
                ),
            )
            .with(
                // TODO No actual delete (since no actual save) at present
                workpad_menu.section("1").item(
                    menu::item(t!("Menus.Workpad.PadDelete"))
                        .shortcut(command(key(keyboard::KeyCode::W)))
                        .on_select(WorkpadMessage::PadClose),
                ),
            )
            .with(
                workpad_menu.section("1").item(
                    menu::item(t!("Menus.Workpad.PadClose"))
                        .shortcut(command(key(keyboard::KeyCode::W)))
                        .on_select(WorkpadMessage::PadClose),
                ),
            )
            .extend(match &self.state {
                State::ActiveSheet(ui) => ui.menu_paths(),
                State::AddSheet(ui) => ui.menu_paths().map(AddSheetMessage::map_to_workpad),
            })
    }

    pub(crate) fn subscription(&self) -> iced::Subscription<WorkpadMessage> {
        match &self.modal {
            ShowModal::None => match self.state {
                State::ActiveSheet(_) => Subscription::none(),
                State::AddSheet(ref ui) => ui.subscription().map(AddSheetMessage::map_to_workpad),
            },
            ShowModal::PadProperties(props) => props
                .subscription()
                .map(PadPropertiesMessage::map_to_workpad),
            ShowModal::SheetProperties(props) => props
                .subscription()
                .map(SheetPropertiesMessage::map_to_workpad),
            ShowModal::ShowError(ui) => ui.subscription().map(|m| match m {
                super::util::error::Message::Acknowledge => WorkpadMessage::ModalCancel,
            }),
        }
    }

    pub fn update(&mut self, message: WorkpadMessage) -> Command<WorkpadMessage> {
        match message {
            WorkpadMessage::NoOp => Command::none(),
            WorkpadMessage::OpenMenu(_) => {
                debug!(target: "flexpad", %message);
                Command::none()
            }
            WorkpadMessage::Multi(messages) => {
                let mut commands = vec![];
                for m in messages {
                    commands.push(self.update(m));
                }
                Command::batch(commands)
            }
            // Modal screens
            WorkpadMessage::ModalCancel => {
                debug!(target: "flexpad", %message);
                self.modal = ShowModal::None;
                Command::none()
            }
            WorkpadMessage::ModalSubmit => {
                debug!(target: "flexpad", %message);
                let mut modal = ShowModal::None;
                std::mem::swap(&mut modal, &mut self.modal);
                self.update_pad(modal.into_update())
            }
            WorkpadMessage::PadPropertiesMsg(msg) => self.modal.update_pad_properties(msg),
            WorkpadMessage::SheetPropertiesMsg(msg) => self.modal.update_sheet_properties(msg),
            // Sub views
            WorkpadMessage::ActiveSheetMsg(msg) => {
                if let ActiveSheetMessage::ViewportChanged(viewport) = msg {
                    let sheet = self.pad.active_sheet().unwrap();
                    self.sheet_viewports.insert(sheet.id(), viewport);
                }
                self.state.update_active_sheet(msg)
            }
            WorkpadMessage::AddSheetMsg(msg) => self.state.update_add_sheet(msg),
            WorkpadMessage::AddSheetCancel => {
                // Can only cancel if there are sheets present
                let sheet = self.pad.active_sheet().unwrap();
                let viewport = self.sheet_viewports.get(&sheet.id()).copied();
                self.state = State::ActiveSheet(ActiveSheetUi::new(sheet, viewport));
                Command::none()
            }
            // Pad actions
            WorkpadMessage::PadUpdated(ref result) => match result {
                Ok(pad) => {
                    debug!(target: "flexpad", %message);
                    self.pad = pad.clone();
                    self.state = match pad.active_sheet() {
                        Some(_) => {
                            let sheet = self.pad.active_sheet().unwrap();
                            let viewport = self.sheet_viewports.get(&sheet.id()).copied();
                            State::ActiveSheet(ActiveSheetUi::new(sheet, viewport))
                        }
                        None => State::AddSheet(AddSheetUi::new(pad.clone())),
                    };
                    get_viewport(active_sheet::GRID_SCROLLABLE_ID.clone())
                        .map(ActiveSheetMessage::ViewportChanged)
                        .map(ActiveSheetMessage::map_to_workpad)
                }
                Err(err) => {
                    error!(target: "flexpad", msg=%message, "Update");
                    self.modal = ShowModal::ShowError(ErrorUi::new(err.to_string()));
                    Command::none()
                }
            },
            WorkpadMessage::PadClose => unreachable!(),
            WorkpadMessage::PadShowProperties => {
                debug!(target: "flexpad", %message);
                self.modal = ShowModal::PadProperties(PadPropertiesUi::new(self.pad.clone()));
                Command::none()
            }
            WorkpadMessage::PadAddSheet => {
                debug!(target: "flexpad", %message);
                self.state = State::AddSheet(AddSheetUi::new(self.pad.clone()));
                Command::none()
            }
            WorkpadMessage::SheetDelete(id) => {
                debug!(target: "flexpad", %message);
                self.update_pad(WorkpadUpdate::SheetDelete { sheet_id: id })
            }
            WorkpadMessage::SheetShowProperties(id) => {
                debug!(target: "flexpad", %message);
                match self.state {
                    State::ActiveSheet(_) => {
                        let sheet = self.pad.sheet_by_id(id).unwrap();
                        self.modal = ShowModal::SheetProperties(SheetPropertiesUi::new(sheet));
                        Command::none()
                    }
                    State::AddSheet(_) => unreachable!(),
                }
            }
        }
    }

    pub fn update_pad(&mut self, update: WorkpadUpdate) -> Command<WorkpadMessage> {
        Command::perform(
            update_pad(self.pad_master.clone(), update),
            WorkpadMessage::PadUpdated,
        )
    }
}

pub async fn update_pad(mut master: WorkpadMaster, update: WorkpadUpdate) -> UpdateResult {
    info!(target: "flexpad", %update, "Model update");
    master.update(update)
}

struct MenuLeafButtonStyle;
impl button::StyleSheet for MenuLeafButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: style.extended_palette().background.base.text,
            border_radius: [4.0; 4].into(),
            background: Some(Color::TRANSPARENT.into()),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let plt = style.extended_palette();
        button::Appearance {
            background: Some(plt.primary.weak.color.into()),
            text_color: plt.primary.weak.text,
            ..self.active(style)
        }
    }
}
