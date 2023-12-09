use std::{cell::RefCell, collections::HashMap, rc::Rc};

use flexpad_grid::{
    style, Border, Borders, CellRange, ColumnHead, Grid, GridCell, GridCorner, GridScrollable,
    RowCol, RowHead, SumSeq, Viewport,
};
use iced::{
    advanced::{mouse::click, widget},
    alignment, theme,
    widget::{
        button, column, horizontal_rule, horizontal_space, image, row, text, vertical_rule,
        vertical_space,
    },
    Alignment, Color, Command, Element, Length,
};
use once_cell::sync::Lazy;
use rust_i18n::t;
use tracing::debug;

use crate::{
    model::workpad::{Cell, Sheet, SheetId, WorkpadUpdate},
    ui::{
        menu,
        util::{images, SPACE_S},
    },
};

use super::{
    active_cell::{self, Editor},
    inactive_cell,
};

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
    NoOp, // Temporary
    SheetShowDetails,
    Focus(widget::Id),
    ViewportChanged(Viewport),
    ActiveCellMove(Move),
    ActiveCellNewValue(String, Move),
    ShowProperties,
    DeleteSheet,
    AddSheet,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("active_sheet::Message::")?;
        match self {
            Self::NoOp => write!(f, "NoOp"),
            Self::SheetShowDetails => write!(f, "SheetShowDetails"),
            Self::Focus(id) => write!(f, "Focus({id:?})"),
            Self::ViewportChanged(viewport) => write!(f, "ViewportChanged({viewport})"),
            Self::ActiveCellMove(mve) => write!(f, "ActiveCellMove({mve})"),
            Self::ActiveCellNewValue(value, mve) => write!(f, "ActiveCellNewValue({value}, {mve})"),
            Self::ShowProperties => write!(f, "EditProperties"),
            Self::DeleteSheet => write!(f, "DeleteSheet"),
            Self::AddSheet => write!(f, "AddSheet"),
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
        let max_column = columns_count.saturating_sub(1) as u32;
        let max_row = rows_count.saturating_sub(1) as u32;
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
    EditSheetPropertiesRequested(Sheet),
    AddSheetRequested,
    UpdateRequested(crate::model::workpad::WorkpadMaster, WorkpadUpdate),
}

#[derive(Debug)]
pub struct ActiveSheetUi {
    pub(crate) active_sheet: Sheet,
    visible_cells: CellRange,
    active_cell: Option<(RowCol, Rc<RefCell<active_cell::Editor>>)>,
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
            let active_cell = RowCol::new(cell.row().index() as u32, cell.column().index() as u32);
            (active_cell, active_cell_editor)
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

    fn active_cell(&self) -> Cell {
        match self.active_cell {
            Some((rc, _)) => self.active_sheet.cell(rc.row as usize, rc.column as usize),
            None => unreachable!(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            self.toolbar_view(),
            vertical_space(SPACE_S),
            horizontal_rule(3),
            self.sheet_and_formula_row_view(),
            self.grid_view(),
        ]
        .align_items(Alignment::Start)
        .into()
    }

    fn toolbar_view(&self) -> iced::Element<'_, Message> {
        let button = |img, _msg| {
            button(image(img))
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Secondary)
        };

        row![
            button(images::undo(), Message::NoOp),
            button(images::redo(), Message::NoOp),
            button(images::print(), Message::NoOp),
            vertical_rule(3),
            button(images::settings(), Message::NoOp),
        ]
        .height(20)
        .spacing(SPACE_S)
        .into()
    }

    fn sheet_and_formula_row_view(&self) -> iced::Element<'_, Message> {
        let button = |img, msg| {
            button(image(img))
                .on_press(msg)
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Text)
        };

        let sheet: iced::Element<'_, Message> = row![
            text(self.active_sheet.name()).size(14).width(200),
            // TODO
            button(images::expand_more(), Message::SheetShowDetails),
        ]
        .spacing(SPACE_S)
        .into();

        match self.active_cell {
            Some((_, ref editor)) => {
                let active_cell = self.active_cell();
                let cell_name: iced::Element<'_, Message> = text(active_cell.name())
                    .size(14)
                    .width(100)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into();

                let formula: iced::Element<'_, Message> =
                    active_cell::ActiveCell::new(editor.clone())
                        .id(FORMULA_BAR_ID.clone())
                        .focused(self.focus == FORMULA_BAR_ID.clone().into())
                        .horizontal_alignment(alignment::Horizontal::Left)
                        .vertical_alignment(alignment::Vertical::Center)
                        .font_size(14)
                        .into();

                row![
                    vertical_rule(3),
                    sheet,
                    vertical_rule(3),
                    cell_name,
                    vertical_rule(3),
                    image(images::fx()).height(20).width(20),
                    formula,
                    vertical_rule(3),
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
        .height(20)
        .spacing(SPACE_S)
        .into()
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
            .push_corner(GridCorner::new(text(t!("ActiveSheet.Corner"))))
            .row_head_width(active_sheet.row_header_width())
            .column_head_height(active_sheet.column_header_height());

        for cl in self.visible_cells.columns() {
            let column = active_sheet.column(cl as usize);
            grid = grid.push_column_head(ColumnHead::new(cl, text(column.name()).size(10)))
        }

        for rw in self.visible_cells.rows() {
            let row = active_sheet.row(rw as usize);
            grid = grid.push_row_head(RowHead::new(rw, text(row.name()).size(10)))
        }

        for rc in self.visible_cells.cells() {
            let RowCol {
                row: rw,
                column: cl,
            } = rc;
            if let Some((rc, _)) = self.active_cell {
                if rc.row != rw || rc.column != cl {
                    let cell = active_sheet.cell(rw as usize, cl as usize);
                    let ic = inactive_cell::InactiveCell::new(rc, cell.value())
                        // TODO Set details from spreadsheet data
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center)
                        .font_size(10.0);

                    let grid_cell = GridCell::new((rw, cl), ic);
                    grid = grid.push_cell(grid_cell);
                }
            };
        }

        if let Some((rc, ref editor)) = self.active_cell {
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
            Message::NoOp => Event::None,
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
                match apply_move(self.active_cell(), mve) {
                    Some((_new_rc, update_active_cell)) => {
                        // TODO Can we issue the ensure_cell_visible when the pad update comes through?
                        // ensure_cell_visible(new_rc)
                        //     .map(super::Message::ActiveSheet)
                        //     .map(ui::Message::Workpad)
                        Event::UpdateRequested(
                            self.active_sheet.workpad().master(),
                            update_active_cell,
                        )
                    }
                    None => Event::None,
                }
            }
            Message::ActiveCellNewValue(ref new_value, mve) => {
                debug!(target: "flexpad", %message);
                let cell = self.active_cell();
                let update_cell_value = WorkpadUpdate::SheetSetCellValue {
                    sheet_id: cell.sheet().id(),
                    row_id: cell.row().id(),
                    column_id: cell.column().id(),
                    value: new_value.clone(),
                };

                match apply_move(cell, mve) {
                    Some((_new_rc, update_active_cell)) => {
                        let update =
                            WorkpadUpdate::Multi(vec![update_cell_value, update_active_cell]);

                        // TODO Can we issue the ensure_cell_visible when the pad update comes through?
                        // ensure_cell_visible(new_rc)
                        //     .map(super::Message::ActiveSheet)
                        //     .map(ui::Message::Workpad)

                        Event::UpdateRequested(self.active_sheet.workpad().master(), update)
                    }
                    None => Event::UpdateRequested(
                        self.active_sheet.workpad().master(),
                        update_cell_value,
                    ),
                }
            }
            Message::ShowProperties => {
                Event::EditSheetPropertiesRequested(self.active_sheet.clone())
            }
            Message::DeleteSheet => Event::UpdateRequested(
                self.active_sheet.workpad().master(),
                WorkpadUpdate::SheetDelete {
                    sheet_id: self.active_sheet.id(),
                },
            ),
            Message::AddSheet => Event::AddSheetRequested,
        }
    }

    pub fn menu_paths(&self) -> menu::PathVec<Message> {
        menu::PathVec::new()
            .with(sheets_menu::show_properties(Some(Message::ShowProperties)))
            .with(sheets_menu::new_sheet(Some(Message::AddSheet)))
            .with(sheets_menu::delete_sheet(Some(Message::DeleteSheet)))
    }
}

pub fn get_viewport() -> Command<Message> {
    flexpad_grid::scroll::get_viewport(GRID_SCROLLABLE_ID.clone()).map(Message::ViewportChanged)
}

pub fn _ensure_cell_visible(cell: RowCol) -> Command<Message> {
    flexpad_grid::scroll::ensure_cell_visible(GRID_SCROLLABLE_ID.clone(), cell)
        .map(Message::ViewportChanged)
}

fn apply_move(active_cell: Cell, mve: Move) -> Option<(RowCol, WorkpadUpdate)> {
    let sheet = active_cell.sheet();
    let prior_rc = rc_of_cell(active_cell);
    let new_rc = mve.apply(prior_rc, sheet.rows().count(), sheet.columns().count());

    if prior_rc != new_rc {
        let new_cell = cell_by_rc(sheet, new_rc);

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

fn rc_of_cell(cell: Cell) -> RowCol {
    RowCol::new(cell.row().index() as u32, cell.column().index() as u32)
}

fn cell_by_rc(sheet: Sheet, rc: RowCol) -> Cell {
    sheet.cell(rc.row as usize, rc.column as usize)
}

mod sheets_menu {
    use iced::keyboard;
    use rust_i18n::t;

    use crate::ui::util::{
        key::{alt, key},
        menu,
    };

    fn root<Message>() -> menu::PathToMenu<Message>
    where
        Message: Clone,
    {
        menu::root(t!("Menus.Sheets.Title"))
    }

    pub fn show_properties<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(
            root(),
            t!("Menus.Sheets.SheetShowProperties"),
            Some(alt(key(keyboard::KeyCode::Comma))),
            on_select,
        )
    }

    pub fn new_sheet<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(
            root(),
            t!("Menus.Sheets.SheetNew"),
            Some(alt(key(keyboard::KeyCode::N))),
            on_select,
        )
    }

    pub fn delete_sheet<Message>(on_select: Option<Message>) -> menu::Path<Message>
    where
        Message: Clone,
    {
        menu::Path::new(
            root(),
            t!("Menus.Sheets.SheetDelete"),
            Some(alt(key(keyboard::KeyCode::Delete))),
            on_select,
        )
    }
}
