use std::{cell::RefCell, rc::Rc};

use flexpad_grid::{
    scroll::ensure_cell_visible, style, Border, Borders, CellRange, ColumnHead, Grid, GridCell,
    GridCorner, GridScrollable, RowCol, RowHead, SumSeq, Viewport,
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
    display_iter,
    model::workpad::{Cell, Sheet, UpdateError, Workpad, WorkpadUpdate},
    ui::{images, SPACE_S},
};

use super::{
    active_cell::{self, Editor},
    inactive_cell, WorkpadMessage,
};

static FORMULA_BAR_ID: Lazy<active_cell::Id> = Lazy::new(active_cell::Id::unique);
static ACTIVE_CELL_ID: Lazy<active_cell::Id> = Lazy::new(active_cell::Id::unique);

pub static GRID_SCROLLABLE_ID: Lazy<flexpad_grid::scroll::Id> =
    Lazy::new(flexpad_grid::scroll::Id::unique);

#[derive(Debug, Clone)]
pub enum ActiveSheetMessage {
    NoOp, // Temporary
    Multi(Vec<ActiveSheetMessage>),
    PadUpdated(Result<Workpad, UpdateError>),
    SheetShowDetails,
    Focus(widget::Id),
    ViewportChanged(Viewport),
    ActiveCellMove(Move),
    ActiveCellNewValue(String),
}

impl ActiveSheetMessage {
    pub fn map_to_workpad(self) -> WorkpadMessage {
        match self {
            Self::PadUpdated(result) => WorkpadMessage::PadUpdated(result),
            m => WorkpadMessage::ActiveSheetMsg(m),
        }
    }
}

impl std::fmt::Display for ActiveSheetMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActiveSheetMessage::")?;
        match self {
            Self::NoOp => write!(f, "NoOp"),
            Self::Multi(msgs) => {
                f.write_str("Multi(")?;
                display_iter(msgs.iter(), f)?;
                f.write_str(")")
            }
            Self::PadUpdated(Ok(workpad)) => write!(f, "PadUpdated({workpad})"),
            Self::PadUpdated(Err(err)) => write!(f, "PadUpdated(ERROR: {err})"),
            Self::SheetShowDetails => write!(f, "SheetShowDetails"),
            Self::Focus(id) => write!(f, "Focus({id:?})"),
            Self::ViewportChanged(viewport) => write!(f, "ViewportChanged({viewport})"),
            Self::ActiveCellMove(mve) => write!(f, "ActiveCellMove({mve})"),
            Self::ActiveCellNewValue(value) => write!(f, "ActiveCellNewValue({value})"),
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

// TODO Flicker when moving/updating
#[derive(Debug)]
pub struct ActiveSheetUi {
    pub(crate) active_sheet: Sheet,
    visible_cells: CellRange,
    active_cell: Option<(RowCol, Rc<RefCell<active_cell::Editor>>)>,
    focus: widget::Id,
}

impl ActiveSheetUi {
    pub fn new(active_sheet: Sheet, viewport: Option<Viewport>) -> Self {
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

    pub fn view(&self) -> iced::Element<'_, ActiveSheetMessage> {
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

    fn toolbar_view(&self) -> iced::Element<'_, ActiveSheetMessage> {
        let button = |img, _msg| {
            button(image(img))
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Secondary)
        };

        row![
            button(images::undo(), ActiveSheetMessage::NoOp),
            button(images::redo(), ActiveSheetMessage::NoOp),
            button(images::print(), ActiveSheetMessage::NoOp),
            vertical_rule(3),
            button(images::settings(), ActiveSheetMessage::NoOp),
        ]
        .height(20)
        .spacing(SPACE_S)
        .into()
    }

    fn sheet_and_formula_row_view(&self) -> iced::Element<'_, ActiveSheetMessage> {
        let button = |img, msg| {
            button(image(img))
                .on_press(msg)
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Text)
        };

        let sheet: iced::Element<'_, ActiveSheetMessage> = row![
            text(self.active_sheet.name()).size(14).width(200),
            // TODO
            button(images::expand_more(), ActiveSheetMessage::SheetShowDetails),
        ]
        .spacing(SPACE_S)
        .into();

        match self.active_cell {
            Some((_, ref editor)) => {
                let active_cell = self.active_cell();
                let cell_name: iced::Element<'_, ActiveSheetMessage> = text(active_cell.name())
                    .size(14)
                    .width(100)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into();

                let formula: iced::Element<'_, ActiveSheetMessage> =
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

    fn grid_view(&self) -> Element<'_, ActiveSheetMessage> {
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
        let mut grid: Grid<ActiveSheetMessage> = Grid::new(heights, widths)
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
            .on_viewport_change(ActiveSheetMessage::ViewportChanged)
            .into()
    }

    pub fn update(&mut self, message: ActiveSheetMessage) -> Command<ActiveSheetMessage> {
        match message {
            ActiveSheetMessage::NoOp => Command::none(),
            ActiveSheetMessage::Multi(messages) => {
                debug!(target: "flexpad", "MULTI");
                let mut commands = vec![];
                for m in messages {
                    commands.push(self.update(m));
                }
                Command::batch(commands)
            }
            ActiveSheetMessage::PadUpdated(_) => unreachable!(),
            ActiveSheetMessage::SheetShowDetails => {
                debug!(target: "flexpad", %message);
                dbg!("Show sheet details");
                Command::none()
            }
            ActiveSheetMessage::Focus(ref id) => {
                // TODO check for edit in progress?
                debug!(target: "flexpad", %message);
                self.focus = id.clone();
                Command::none()
            }
            ActiveSheetMessage::ViewportChanged(viewport) => {
                debug!(target: "flexpad", %message);
                self.visible_cells = viewport.cell_range();
                Command::none()
            }
            ActiveSheetMessage::ActiveCellMove(mve) => {
                debug!(target:"flexpad", %message);
                let mut commands = vec![];
                let sheet = &self.active_sheet;
                // Cannot move active cell unless there is one to move it from
                let (prior_active_cell, prior_editor) = self.active_cell.as_ref().unwrap();
                let RowCol { row, column } = *prior_active_cell;
                let mut new_active_cell = *prior_active_cell;
                match mve {
                    Move::Left => {
                        if column > 0 {
                            new_active_cell = RowCol::new(row, column - 1);
                        }
                    }
                    Move::Right => {
                        let columns_count = sheet.columns().count() as u32;
                        let max_index = columns_count.max(1) - 1;
                        if column < max_index {
                            new_active_cell = RowCol::new(row, column + 1);
                        }
                    }
                    Move::Up => {
                        if row > 0 {
                            new_active_cell = RowCol::new(row - 1, column);
                        }
                    }
                    Move::Down => {
                        let rows_count = sheet.rows().count() as u32;
                        let max_index = rows_count.max(1) - 1;
                        if row < max_index {
                            new_active_cell = RowCol::new(row + 1, column);
                        }
                    }
                    Move::JumpLeft => {
                        new_active_cell = RowCol::new(row, 0);
                    }
                    Move::JumpRight => {
                        let columns_count = sheet.columns().count() as u32;
                        let max_index = columns_count.max(1) - 1;
                        new_active_cell = RowCol::new(row, max_index);
                    }
                    Move::JumpUp => {
                        new_active_cell = RowCol::new(0, column);
                    }
                    Move::JumpDown => {
                        let rows_count = sheet.rows().count() as u32;
                        let max_index = rows_count.max(1) - 1;
                        new_active_cell = RowCol::new(max_index, column);
                    }
                    Move::To(rc) => {
                        new_active_cell = rc;
                    }
                }

                if *prior_active_cell != new_active_cell {
                    let rw = new_active_cell.row as usize;
                    let cl = new_active_cell.column as usize;
                    let cell = sheet.cell(rw, cl);

                    commands.push(
                        ensure_cell_visible(GRID_SCROLLABLE_ID.clone(), new_active_cell)
                            .map(ActiveSheetMessage::ViewportChanged),
                    );

                    let update_cell_value = if prior_editor.borrow().is_editing() {
                        let cell = self.active_cell();
                        Some(WorkpadUpdate::SheetSetCellValue {
                            sheet_id: cell.sheet().id(),
                            row_id: cell.row().id(),
                            column_id: cell.column().id(),
                            value: prior_editor.borrow().contents(),
                        })
                    } else {
                        None
                    };

                    let update_active_cell = WorkpadUpdate::SheetSetActiveCell {
                        sheet_id: cell.sheet().id(),
                        row_id: cell.row().id(),
                        column_id: cell.column().id(),
                    };

                    let update = match update_cell_value {
                        Some(update_cell_value) => {
                            WorkpadUpdate::Multi(vec![update_cell_value, update_active_cell])
                        }
                        None => update_active_cell,
                    };
                    commands.push(self.update_pad(update));
                }

                if commands.is_empty() {
                    Command::none()
                } else {
                    Command::batch(commands)
                }
            }
            ActiveSheetMessage::ActiveCellNewValue(ref new_value) => {
                debug!(target: "flexpad", %message);
                let cell = self.active_cell();
                let update = WorkpadUpdate::SheetSetCellValue {
                    sheet_id: cell.sheet().id(),
                    row_id: cell.row().id(),
                    column_id: cell.column().id(),
                    value: new_value.clone(),
                };

                self.update_pad(update)
            }
        }
    }

    pub fn update_pad(&mut self, update: WorkpadUpdate) -> Command<ActiveSheetMessage> {
        Command::perform(
            super::update_pad(self.active_sheet.workpad().master(), update),
            ActiveSheetMessage::PadUpdated,
        )
    }
}
