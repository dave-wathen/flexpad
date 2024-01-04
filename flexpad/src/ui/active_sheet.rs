use super::util::{FlexpadAction, ICON_OPEN_DOWN};
use crate::ui::{
    edit_menu,
    util::{ICON_FX, TEXT_SIZE_LABEL},
    widget::{
        active_cell::{self, Editor},
        inactive_cell,
    },
    workpad_menu,
};
use flexpad_grid::{
    style, Border, Borders, CellRange, ColumnHead, Grid, GridCell, GridCorner, GridScrollable,
    RowCol, RowHead, SumSeq, Viewport,
};
use flexpad_model::{Cell, Sheet, SheetId, Version, Workpad, WorkpadMaster, WorkpadUpdate};
use flexpad_toolkit::{menu, prelude::*, toolbar::Toolbar};
use iced::{
    advanced::{mouse::click, widget},
    alignment, theme,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, row, text, vertical_rule,
    },
    Alignment, Color, Command, Element, Length, Pixels, Subscription,
};
use once_cell::sync::Lazy;
use rust_i18n::t;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use tracing::debug;

static FORMULA_BAR_ID: Lazy<active_cell::Id> = Lazy::new(active_cell::Id::unique);
static ACTIVE_CELL_ID: Lazy<active_cell::Id> = Lazy::new(active_cell::Id::unique);

pub static GRID_SCROLLABLE_ID: Lazy<flexpad_grid::scroll::Id> =
    Lazy::new(flexpad_grid::scroll::Id::unique);

thread_local! {
    static VIEWPORTS_CACHE: RefCell<HashMap<(String, SheetId), Viewport>> =
        RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub enum Message {
    Focus(widget::Id),
    ViewportChanged(Viewport),
    ActiveCellMove(Move),
    ActiveCellNewValue(String, Move),
    SheetShowDetails,
    SheetShowProperties,
    SheetDelete,
    SheetAdd,
    PadClose,
    PadShowProperties,
    SetActiveSheet(SheetId),
    GotoVersion(Version),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("active_sheet::Message::")?;
        match self {
            Self::SheetShowDetails => write!(f, "SheetShowDetails"),
            Self::Focus(id) => write!(f, "Focus({id:?})"),
            Self::ViewportChanged(viewport) => write!(f, "ViewportChanged({viewport})"),
            Self::ActiveCellMove(mve) => write!(f, "ActiveCellMove({mve})"),
            Self::ActiveCellNewValue(value, mve) => write!(f, "ActiveCellNewValue({value}, {mve})"),
            Self::SheetShowProperties => write!(f, "EditProperties"),
            Self::SheetDelete => write!(f, "DeleteSheet"),
            Self::SheetAdd => write!(f, "AddSheet"),
            Self::PadShowProperties => write!(f, "PadShowProperties"),
            Self::PadClose => write!(f, "PadClose"),
            Self::SetActiveSheet(id) => write!(f, "SetActiveSheet({id})"),
            Self::GotoVersion(version) => write!(f, "GotoVersion({version})"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
    JumpLeft,
    JumpRight,
    JumpUp,
    JumpDown,
    To(RowCol),
}

impl Move {
    fn apply(&self, position: RowCol, rows_count: usize, columns_count: usize) -> RowCol {
        let RowCol { row, column } = position;
        let max_column = columns_count.saturating_sub(1);
        let max_row = rows_count.saturating_sub(1);
        match self {
            Move::Left => RowCol::new(row, column.saturating_sub(1)),
            Move::Right => RowCol::new(row, (column + 1).min(max_column)),
            Move::Up => RowCol::new(row.saturating_sub(1), column),
            Move::Down => RowCol::new((row + 1).min(max_row), column),
            Move::JumpLeft => RowCol::new(row, 0),
            Move::JumpRight => RowCol::new(row, max_column),
            Move::JumpUp => RowCol::new(0, column),
            Move::JumpDown => RowCol::new(max_row, column),
            Move::To(rc) => *rc,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Left => write!(f, "Left"),
            Move::Right => write!(f, "Right"),
            Move::Up => write!(f, "Up"),
            Move::Down => write!(f, "Down"),
            Move::JumpLeft => write!(f, "JumpLeft"),
            Move::JumpRight => write!(f, "JumpRight"),
            Move::JumpUp => write!(f, "JumpUp"),
            Move::JumpDown => write!(f, "JumpDown"),
            Move::To(cell) => write!(f, "To({cell}"),
        }
    }
}

pub enum Event {
    None,
    EditPadPropertiesRequested(Workpad),
    CloseWorkpadRequested,
    EditSheetPropertiesRequested(Sheet),
    AddSheetRequested(Workpad),
    UpdateRequested(WorkpadMaster, WorkpadUpdate),
}

#[derive(Debug)]
pub struct ActiveSheetUi {
    pub(crate) active_sheet: Sheet,
    visible_cells: CellRange,
    active_cell: Option<(Cell, Rc<RefCell<active_cell::Editor>>)>,
    focus: widget::Id,
}

impl ActiveSheetUi {
    pub fn new(active_sheet: Sheet) -> Self {
        let workpad_id = String::from(active_sheet.workpad().id());
        let viewport = VIEWPORTS_CACHE.with(|cache| {
            cache
                .borrow()
                .get(&(workpad_id, active_sheet.id()))
                .copied()
        });

        let active_cell = active_sheet.active_cell().map(|cell| {
            let active_cell_editor = Rc::new(RefCell::new(Editor::new(cell.value())));
            (cell, active_cell_editor)
        });

        let visible_cells = match viewport {
            Some(viewport) => viewport.cell_range(),
            None => CellRange::empty(),
        };

        Self {
            active_sheet,
            visible_cells,
            active_cell,
            focus: ACTIVE_CELL_ID.clone().into(),
        }
    }

    pub fn title(&self) -> String {
        self.active_sheet.workpad().name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            self.toolbar_view(),
            self.sheet_and_formula_row_view(),
            self.grid_view(),
        ]
        .align_items(Alignment::Start)
        .into()
    }

    fn toolbar_view(&self) -> iced::Element<'_, Message> {
        let (undo_to, redo_to) = surrounding_versions(&self.active_sheet.workpad());

        Toolbar::new()
            .action(FlexpadAction::Undo, undo_to.map(Message::GotoVersion))
            .action(FlexpadAction::Redo, redo_to.map(Message::GotoVersion))
            .action(FlexpadAction::Print, None)
            .separator()
            .action(FlexpadAction::Properties, None)
            .into()
    }

    fn sheet_and_formula_row_view(&self) -> iced::Element<'_, Message> {
        const TEXT_SIZE: Pixels = TEXT_SIZE_LABEL;

        let button = |img, msg| {
            button(img)
                .on_press(msg)
                .padding(SPACE_S)
                .width(Length::Shrink)
                .style(theme::Button::Text)
        };

        let sheet: iced::Element<'_, Message> = row![
            text(self.active_sheet.name())
                .size(TEXT_SIZE)
                .line_height(1.0)
                .height(Length::Fill)
                .vertical_alignment(alignment::Vertical::Center)
                .width(200),
            // TODO
            button(icon(ICON_OPEN_DOWN, TEXT_SIZE), Message::SheetShowDetails),
        ]
        .spacing(SPACE_S)
        .into();

        let controls = match self.active_cell {
            Some((_, ref editor)) => {
                let Some((active_cell, _)) = &self.active_cell else {
                    unreachable!();
                };
                let cell_name: iced::Element<'_, Message> = text(active_cell.name())
                    .size(TEXT_SIZE)
                    .line_height(1.0)
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .width(100)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into();

                let formula: iced::Element<'_, Message> =
                    active_cell::ActiveCell::new(editor.clone())
                        .id(FORMULA_BAR_ID.clone())
                        .focused(self.focus == FORMULA_BAR_ID.clone().into())
                        .horizontal_alignment(alignment::Horizontal::Left)
                        .vertical_alignment(alignment::Vertical::Center)
                        .font_size(TEXT_SIZE)
                        .into();

                row![
                    vertical_rule(1),
                    sheet,
                    vertical_rule(1),
                    cell_name,
                    vertical_rule(1),
                    container(icon(ICON_FX, TEXT_SIZE)).height(Length::Fill),
                    formula,
                    vertical_rule(1),
                ]
            }
            None => {
                row![
                    vertical_rule(3),
                    sheet,
                    vertical_rule(3),
                    horizontal_space(Length::Fill)
                ]
            }
        }
        .height(SPACE_S + TEXT_SIZE.0 + SPACE_S)
        .spacing(SPACE_S);

        column![horizontal_rule(1), controls].into()
    }

    fn grid_view(&self) -> Element<'_, Message> {
        let active_sheet = &self.active_sheet;
        // TODO Allow hetrogenious sizes
        let mut widths = SumSeq::new();
        widths.push_many(
            active_sheet.columns().count() as u32,
            active_sheet.columns().next().unwrap().width(),
        );
        let mut heights = SumSeq::new();
        heights.push_many(
            active_sheet.rows().count() as u32,
            active_sheet.rows().next().unwrap().height(),
        );

        // TODO Hardcoded text sizes
        let mut grid: Grid<Message> = Grid::new(heights, widths)
            .style(style::Grid::Ruled)
            .push_corner(GridCorner::new(
                text(t!("ActiveSheet.Corner")).size(12).line_height(1.0),
            ))
            .row_head_width(active_sheet.row_header_width())
            .column_head_height(active_sheet.column_header_height());

        for cl in self.visible_cells.columns() {
            let column = active_sheet.column(cl);
            grid = grid.push_column_head(ColumnHead::new(
                cl,
                text(column.name()).size(12).line_height(1.0),
            ))
        }

        for rw in self.visible_cells.rows() {
            let row = active_sheet.row(rw);
            grid = grid.push_row_head(RowHead::new(rw, text(row.name()).size(12).line_height(1.0)))
        }

        let active_cell_rc = self.active_cell.as_ref().map(|(cell, _)| rc_of_cell(cell));
        for rc in self.visible_cells.cells() {
            if Some(rc) != active_cell_rc {
                let cell = cell_by_rc(active_sheet, rc);
                let ic = inactive_cell::InactiveCell::new(rc, cell.value())
                    // TODO Set details from spreadsheet data
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center)
                    .font_size(10.0);

                let grid_cell = GridCell::new(rc, ic);
                grid = grid.push_cell(grid_cell);
            };
        }

        if let Some((cell, editor)) = &self.active_cell {
            let rc = rc_of_cell(cell);
            // Always add the active cell even when not visible so keystrokes are handled
            let ac = active_cell::ActiveCell::new(editor.clone())
                .id(ACTIVE_CELL_ID.clone())
                .focused(self.focus == ACTIVE_CELL_ID.clone().into())
                .edit_when_clicked(click::Kind::Double)
                // TODO Set details from spreadsheet data
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center)
                .font_size(10.0);
            let grid_cell = GridCell::new(rc, ac)
                // TODO Hardcoding
                .borders(Borders::new(Border::new(1.0, Color::from_rgb8(0, 0, 255))));
            grid = grid.push_cell(grid_cell);
        }

        GridScrollable::new(grid)
            .id(GRID_SCROLLABLE_ID.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_viewport_change(Message::ViewportChanged)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::SheetShowDetails => {
                debug!(target: "flexpad", %message);
                dbg!("Show sheet details");
                Event::None
            }
            Message::Focus(ref id) => {
                // TODO check for edit in progress?
                debug!(target: "flexpad", %message);
                self.focus = id.clone();
                Event::None
            }
            Message::ViewportChanged(viewport) => {
                debug!(target: "flexpad", %message);
                let workpad_id = String::from(self.active_sheet.workpad().id());
                VIEWPORTS_CACHE.with(|cache| {
                    cache
                        .borrow_mut()
                        .insert((workpad_id, self.active_sheet.id()), viewport)
                });
                self.visible_cells = viewport.cell_range();
                Event::None
            }
            Message::ActiveCellMove(mve) => {
                debug!(target:"flexpad", %message);
                let Some((_, editor)) = &self.active_cell else {
                    unreachable!();
                };
                let mut editor = editor.borrow_mut();
                let new_value = editor.is_editing().then(|| editor.end_editing());
                self.update_value_and_move(new_value, mve)
            }
            Message::ActiveCellNewValue(ref new_value, mve) => {
                debug!(target: "flexpad", %message);
                self.update_value_and_move(Some(new_value.clone()), mve)
            }
            Message::SheetShowProperties => {
                Event::EditSheetPropertiesRequested(self.active_sheet.clone())
            }
            Message::SheetDelete => Event::UpdateRequested(
                self.active_sheet.workpad().master(),
                WorkpadUpdate::SheetDelete {
                    sheet_id: self.active_sheet.id(),
                },
            ),
            Message::SheetAdd => Event::AddSheetRequested(self.active_sheet.workpad()),
            Message::PadShowProperties => {
                Event::EditPadPropertiesRequested(self.active_sheet.workpad())
            }
            Message::PadClose => Event::CloseWorkpadRequested,
            Message::SetActiveSheet(sheet_id) => Event::UpdateRequested(
                self.active_sheet.workpad().master(),
                WorkpadUpdate::SetActiveSheet { sheet_id },
            ),
            Message::GotoVersion(version) => Event::UpdateRequested(
                self.active_sheet.workpad().master(),
                WorkpadUpdate::SetVersion { version },
            ),
        }
    }

    fn update_value_and_move(&self, new_value: Option<String>, mve: Move) -> Event {
        let Some((cell, _)) = &self.active_cell else {
            unreachable!();
        };

        let update_cell_value = new_value.map(|new_value| WorkpadUpdate::SheetSetCellValue {
            sheet_id: cell.sheet().id(),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
            value: new_value,
        });

        let update_active_cell = apply_move(cell, mve).map(|(_, update)| update);

        let master = self.active_sheet.workpad().master();
        match (update_cell_value, update_active_cell) {
            (None, None) => Event::None,
            (None, Some(update_active_cell)) => Event::UpdateRequested(master, update_active_cell),
            (Some(update_cell_value), None) => Event::UpdateRequested(master, update_cell_value),
            (Some(update_cell_value), Some(update_active_cell)) => Event::UpdateRequested(
                master,
                WorkpadUpdate::Multi(vec![update_cell_value, update_active_cell]),
            ),
        }
    }

    pub fn pad_updated(&mut self, pad: Workpad) -> Command<Message> {
        let new_active_sheet = pad.active_sheet().unwrap();

        if self.active_sheet.id() != new_active_sheet.id() {
            // View has switched to a new sheet
            let viewport = VIEWPORTS_CACHE.with(|cache| {
                cache
                    .borrow()
                    .get(&(
                        new_active_sheet.workpad().id().to_owned(),
                        new_active_sheet.id(),
                    ))
                    .copied()
            });
            self.active_sheet = new_active_sheet;
            self.visible_cells = match viewport {
                Some(viewport) => viewport.cell_range(),
                None => CellRange::empty(),
            };
            self.active_cell = self.active_sheet.active_cell().map(|cell| {
                let active_cell_editor = Rc::new(RefCell::new(Editor::new(cell.value())));
                (cell, active_cell_editor)
            });

            let scroll_to = self.visible_cells.cells().next().map(scroll_to);
            let ensure_visible = self
                .active_cell
                .as_ref()
                .map(|(cell, _)| ensure_cell_visible(rc_of_cell(cell)));

            match (scroll_to, ensure_visible) {
                (None, None) => Command::none(),
                (None, Some(ensure_visible)) => ensure_visible,
                (Some(scroll_to), None) => scroll_to,
                (Some(scroll_to), Some(ensure_visible)) => {
                    Command::batch(vec![scroll_to, ensure_visible])
                }
            }
        } else {
            // View has switched to a new version of the same sheet
            self.active_sheet = new_active_sheet;

            let prior_rc = self.active_cell.as_ref().map(|(cell, _)| rc_of_cell(cell));
            self.active_cell = self.active_sheet.active_cell().map(|cell| {
                let active_cell_editor = Rc::new(RefCell::new(Editor::new(cell.value())));
                (cell, active_cell_editor)
            });
            let new_rc = self.active_cell.as_ref().map(|(cell, _)| rc_of_cell(cell));

            match (prior_rc, new_rc) {
                (Some(prior), Some(new)) if prior != new => ensure_cell_visible(new),
                _ => Command::none(),
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        let (undo_to, redo_to) = surrounding_versions(&self.active_sheet.workpad());

        let mut paths = menu::PathVec::new()
            .with(workpad_menu::new_blank_workpad(None))
            .with(workpad_menu::new_starter_workpad(None))
            .with(workpad_menu::show_properties(Some(
                Message::PadShowProperties,
            )))
            // TODO No actual delete (since no actual save) at present
            .with(workpad_menu::delete_pad(Some(Message::PadClose)))
            .with(workpad_menu::close_pad(Some(Message::PadClose)))
            .with(edit_menu::undo(undo_to.map(Message::GotoVersion)))
            .with(edit_menu::redo(redo_to.map(Message::GotoVersion)))
            .with(sheets_menu::show_properties(Some(
                Message::SheetShowProperties,
            )))
            .with(sheets_menu::new_sheet(Some(Message::SheetAdd)))
            .with(sheets_menu::delete_sheet(Some(Message::SheetDelete)));

        for sheet in self.active_sheet.workpad().sheets() {
            let on_select = if sheet == self.active_sheet {
                None
            } else {
                Some(Message::SetActiveSheet(sheet.id()))
            };
            paths = paths.with(sheets_menu::activate_sheet(
                sheet.name().to_owned(),
                on_select,
            ));
        }

        paths
    }
}

fn surrounding_versions(pad: &Workpad) -> (Option<Version>, Option<Version>) {
    (
        pad.backward_versions().next().map(|version| version.0),
        pad.forward_versions().next().map(|version| version.0),
    )
}

pub fn ensure_cell_visible(cell: RowCol) -> Command<Message> {
    flexpad_grid::scroll::ensure_cell_visible(GRID_SCROLLABLE_ID.clone(), cell)
        .map(Message::ViewportChanged)
}

pub fn scroll_to(cell: RowCol) -> Command<Message> {
    flexpad_grid::scroll::scroll_to_cell(GRID_SCROLLABLE_ID.clone(), cell)
        .map(Message::ViewportChanged)
}

fn apply_move(active_cell: &Cell, mve: Move) -> Option<(RowCol, WorkpadUpdate)> {
    let sheet = active_cell.sheet();
    let prior_rc = rc_of_cell(active_cell);
    let new_rc = mve.apply(prior_rc, sheet.rows().count(), sheet.columns().count());

    if prior_rc != new_rc {
        let new_cell = cell_by_rc(&sheet, new_rc);

        let update_active_cell = WorkpadUpdate::SheetSetActiveCell {
            sheet_id: new_cell.sheet().id(),
            row_id: new_cell.row().id(),
            column_id: new_cell.column().id(),
        };

        Some((new_rc, update_active_cell))
    } else {
        None
    }
}

fn rc_of_cell(cell: &Cell) -> RowCol {
    RowCol::new(cell.row().index(), cell.column().index())
}

fn cell_by_rc(sheet: &Sheet, rc: RowCol) -> Cell {
    sheet.cell(rc.row, rc.column)
}

mod sheets_menu {
    use flexpad_toolkit::{menu, prelude::*};
    use rust_i18n::t;

    use crate::ui::util::FlexpadAction;

    fn root<Message>() -> menu::PathToMenu<Message>
    where
        Message: Clone,
    {
        menu::root(t!("Menu.Sheet"))
    }

    fn activate_sheets<Message>() -> menu::PathToMenuSection<Message>
    where
        Message: Clone,
    {
        root().section("sheets")
    }

    pub fn show_properties<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(root(), FlexpadAction::SheetProperties, on_select)
    }

    pub fn new_sheet<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(root(), FlexpadAction::SheetNew, on_select)
    }

    pub fn delete_sheet<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(root(), FlexpadAction::SheetDelete, on_select)
    }

    pub fn activate_sheet<Message>(name: String, on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(activate_sheets(), Action::new(name), on_select)
    }
}
