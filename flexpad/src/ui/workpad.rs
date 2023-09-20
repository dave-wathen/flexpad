use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use crate::model::workpad::Workpad;
use flexpad_grid::{
    scroll::ensure_cell_visible, style, Border, Borders, CellRange, ColumnHead, Grid, GridCell,
    GridCorner, GridScrollable, RowCol, RowHead, SumSeq, Viewport,
};
use iced::{
    advanced::widget,
    alignment, theme,
    widget::{button, column, horizontal_space, image, row, text, text_input, vertical_rule},
    Alignment, Color, Command, Element, Length,
};

use self::active::{ActiveCell, Editor};

use super::images;

mod active;

use once_cell::sync::Lazy;

static FORMULA_BAR_ID: Lazy<active::Id> = Lazy::new(active::Id::unique);
static ACTIVE_CELL_ID: Lazy<active::Id> = Lazy::new(active::Id::unique);

static GRID_SCROLLABLE_ID: Lazy<flexpad_grid::scroll::Id> =
    Lazy::new(flexpad_grid::scroll::Id::unique);

#[derive(Debug, Default, Clone)]
enum State {
    #[default]
    Passive,
    EditingPadName(String),
    EditingSheetName(String),
}

#[derive(Debug, Clone)]
pub enum WorkpadMessage {
    NoOp, // TODO Temporary
    Multi(Vec<WorkpadMessage>),
    PadClose,
    PadNameEditStart,
    PadNameEdited(String),
    PadNameEditEnd,
    SheetNameEditStart,
    SheetNameEdited(String),
    SheetNameEditEnd,
    Focus(widget::Id),
    ViewportChanged(Viewport),
    ActiveCellMove(Move),
    ActiveCellNewValue(String),
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
}

pub struct WorkpadUI {
    pad: Arc<RwLock<Workpad>>,
    state: State,
    name_edit_id: text_input::Id,
    sheet_edit_id: text_input::Id,
    visible_cells: CellRange,
    active_cell: RowCol,
    active_cell_editor: Rc<RefCell<active::Editor>>,
    focus: widget::Id,
}

impl WorkpadUI {
    pub fn new(pad: Arc<RwLock<Workpad>>) -> Self {
        let active_cell_editor = {
            let pad = pad.read().unwrap();
            let active_sheet = pad.active_sheet();
            let active_cell = active_sheet.cell(0, 0);
            Editor::new(active_cell.value())
        };

        Self {
            pad,
            state: Default::default(),
            name_edit_id: text_input::Id::unique(),
            sheet_edit_id: text_input::Id::unique(),
            visible_cells: CellRange::empty(),
            active_cell: (0, 0).into(),
            active_cell_editor: Rc::new(RefCell::new(active_cell_editor)),
            focus: ACTIVE_CELL_ID.clone().into(),
        }
    }

    pub fn title(&self) -> String {
        self.pad.read().unwrap().name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, WorkpadMessage> {
        column![
            self.name_view(),
            self.toolbar_view(),
            self.sheet_and_formula_row_view(),
            self.grid_view(),
        ]
        .padding(10)
        .spacing(5)
        .align_items(Alignment::Start)
        .into()
    }

    // TODO Row sized increases when switching to edit
    fn name_view(&self) -> iced::Element<'_, WorkpadMessage> {
        let button = |img, msg| {
            button(image(img))
                .on_press(msg)
                .width(Length::Shrink)
                .height(25)
                .padding(2)
                .style(theme::Button::Text)
        };

        match &self.state {
            State::EditingPadName(name) => text_input("Workpad Name", name)
                .id(self.name_edit_id.clone())
                .size(25)
                .on_input(WorkpadMessage::PadNameEdited)
                .on_submit(WorkpadMessage::PadNameEditEnd)
                .into(),
            _ => row![
                text(self.pad.read().unwrap().name()).size(25),
                horizontal_space(5),
                button(images::edit(), WorkpadMessage::PadNameEditStart),
                // TODO No actal delete (since no actual save) at present
                button(images::delete(), WorkpadMessage::PadClose),
                button(images::close(), WorkpadMessage::PadClose),
            ]
            .into(),
        }
    }

    fn toolbar_view(&self) -> iced::Element<'_, WorkpadMessage> {
        let button = |img, _msg| {
            button(image(img))
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Secondary)
        };

        row![
            button(images::undo(), WorkpadMessage::NoOp),
            button(images::redo(), WorkpadMessage::NoOp),
            button(images::print(), WorkpadMessage::NoOp),
            vertical_rule(3),
            button(images::settings(), WorkpadMessage::NoOp),
        ]
        .height(20)
        .spacing(3)
        .into()
    }

    // TODO Cannot see text properly when editing
    fn sheet_and_formula_row_view(&self) -> iced::Element<'_, WorkpadMessage> {
        let button = |img, msg| {
            button(image(img))
                .on_press(msg)
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Text)
        };

        let sheet: iced::Element<'_, WorkpadMessage> = match &self.state {
            State::EditingSheetName(name) => text_input("Sheet name", name)
                .id(self.sheet_edit_id.clone())
                .size(20)
                .on_input(WorkpadMessage::SheetNameEdited)
                .on_submit(WorkpadMessage::SheetNameEditEnd)
                .into(),
            _ => row![
                text(self.pad.read().unwrap().active_sheet().name()).size(18),
                horizontal_space(5),
                button(images::edit(), WorkpadMessage::SheetNameEditStart),
                // TODO
                button(images::delete(), WorkpadMessage::NoOp),
                // TODO
                button(images::expand_more(), WorkpadMessage::NoOp),
            ]
            .width(200)
            .into(),
        };

        let pad = self.pad.read().unwrap();
        let active_sheet = pad.active_sheet();
        let active_cell = active_sheet.cell(
            self.active_cell.row as usize,
            self.active_cell.column as usize,
        );
        let cell_name: iced::Element<'_, WorkpadMessage> = text(active_cell.name())
            .size(14)
            .width(100)
            .horizontal_alignment(alignment::Horizontal::Center)
            .into();

        let formula: iced::Element<'_, WorkpadMessage> =
            ActiveCell::new(self.active_cell_editor.clone())
                .id(FORMULA_BAR_ID.clone())
                .focused(self.focus == FORMULA_BAR_ID.clone().into())
                .horizontal_alignment(alignment::Horizontal::Left)
                .vertical_alignment(alignment::Vertical::Center)
                .font_size(14)
                .into();

        row![
            sheet,
            vertical_rule(3),
            cell_name,
            vertical_rule(3),
            image(images::fx()).height(20).width(20),
            formula
        ]
        .height(20)
        .spacing(5)
        .into()
    }

    fn grid_view(&self) -> Element<'_, WorkpadMessage> {
        let pad = self.pad.read().unwrap();
        let sheet = pad.active_sheet();

        // TODO Allow hetrogenious sizes
        let mut widths = SumSeq::new();
        widths.push_many(
            sheet.columns().count() as u32,
            sheet.columns().next().unwrap().width(),
        );
        let mut heights = SumSeq::new();
        heights.push_many(
            sheet.rows().count() as u32,
            sheet.rows().next().unwrap().height(),
        );

        // TODO Hardcoded text sizes
        let mut grid: Grid<WorkpadMessage> = Grid::new(heights, widths)
            .style(style::Grid::Ruled)
            .push_corner(GridCorner::new(text("*")))
            .row_head_width(sheet.row_header_width())
            .column_head_height(sheet.column_header_height());

        for cl in self.visible_cells.columns() {
            let column = sheet.column(cl as usize);
            grid = grid.push_column_head(ColumnHead::new(cl, text(column.name()).size(10)))
        }

        for rw in self.visible_cells.rows() {
            let row = sheet.row(rw as usize);
            grid = grid.push_row_head(RowHead::new(rw, text(row.name()).size(10)))
        }

        for RowCol {
            row: rw,
            column: cl,
        } in self.visible_cells.cells()
        {
            if self.active_cell.row != rw || self.active_cell.column != cl {
                let cell = sheet.cell(rw as usize, cl as usize);
                let grid_cell = GridCell::new((rw, cl), text(cell.value()).size(10));
                grid = grid.push_cell(grid_cell);
            };
        }

        // Always add the active cell even when not visible so keystrokes are handled
        let ac = ActiveCell::new(self.active_cell_editor.clone())
            .id(ACTIVE_CELL_ID.clone())
            .focused(self.focus == ACTIVE_CELL_ID.clone().into())
            // TODO Set details from spreadsheet data
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center)
            .font_size(10.0);
        let grid_cell = GridCell::new(self.active_cell, ac)
            // TODO Hardcoding
            .borders(Borders::new(Border::new(1.0, Color::from_rgb8(0, 0, 255))));
        grid = grid.push_cell(grid_cell);

        GridScrollable::new(grid)
            .id(GRID_SCROLLABLE_ID.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_viewport_change(WorkpadMessage::ViewportChanged)
            .into()
    }

    // TODO Cancel editing using Esc/Button
    pub fn update(&mut self, message: WorkpadMessage) -> Command<WorkpadMessage> {
        if let WorkpadMessage::Multi(messages) = message {
            let mut commands = vec![];
            for m in messages {
                commands.push(self.update(m));
            }
            return Command::batch(commands);
        }

        if let WorkpadMessage::ViewportChanged(viewport) = message {
            self.visible_cells = viewport.cell_range()
        }

        match &self.state {
            State::Passive => match message {
                WorkpadMessage::Focus(id) => {
                    // TODO check for edit in progress?
                    self.focus = id;
                    Command::none()
                }
                WorkpadMessage::ActiveCellNewValue(s) => {
                    // TODO Move this to model and perform updates (and recaclulations) on another thread
                    let mut pad = self.pad.write().unwrap();
                    let editor = Editor::new(&s);
                    pad.set_cell_value(
                        self.active_cell.row as usize,
                        self.active_cell.column as usize,
                        s,
                    );
                    self.active_cell_editor = Rc::new(RefCell::new(editor));

                    Command::none()
                }
                WorkpadMessage::ActiveCellMove(mve) => {
                    let pad = self.pad.read().unwrap();
                    let sheet = pad.active_sheet();
                    let prior_active_cell = self.active_cell;
                    match mve {
                        Move::Left => {
                            if self.active_cell.column > 0 {
                                self.active_cell =
                                    RowCol::new(self.active_cell.row, self.active_cell.column - 1);
                            }
                        }
                        Move::Right => {
                            let columns_count = sheet.columns().count() as u32;
                            let max_index = columns_count.max(1) - 1;
                            if self.active_cell.column < max_index {
                                self.active_cell =
                                    RowCol::new(self.active_cell.row, self.active_cell.column + 1);
                            }
                        }
                        Move::Up => {
                            if self.active_cell.row > 0 {
                                self.active_cell =
                                    RowCol::new(self.active_cell.row - 1, self.active_cell.column);
                            }
                        }
                        Move::Down => {
                            let rows_count = sheet.rows().count() as u32;
                            let max_index = rows_count.max(1) - 1;
                            if self.active_cell.row < max_index {
                                self.active_cell =
                                    RowCol::new(self.active_cell.row + 1, self.active_cell.column);
                            }
                        }
                        Move::JumpLeft => {
                            if self.active_cell.column > 0 {
                                self.active_cell = RowCol::new(self.active_cell.row, 0);
                            }
                        }
                        Move::JumpRight => {
                            let columns_count = sheet.columns().count() as u32;
                            let max_index = columns_count.max(1) - 1;
                            if self.active_cell.column < max_index {
                                self.active_cell = RowCol::new(self.active_cell.row, max_index);
                            }
                        }
                        Move::JumpUp => {
                            if self.active_cell.row > 0 {
                                self.active_cell = RowCol::new(0, self.active_cell.column);
                            }
                        }
                        Move::JumpDown => {
                            let rows_count = sheet.rows().count() as u32;
                            let max_index = rows_count.max(1) - 1;
                            if self.active_cell.row < max_index {
                                self.active_cell = RowCol::new(max_index, self.active_cell.column);
                            }
                        }
                    }

                    if prior_active_cell != self.active_cell {
                        let rw = self.active_cell.row as usize;
                        let cl = self.active_cell.column as usize;
                        let cell = sheet.cell(rw, cl);
                        let editor = Editor::new(cell.value());
                        self.active_cell_editor = Rc::new(RefCell::new(editor));
                        self.focus = ACTIVE_CELL_ID.clone().into();
                    }

                    ensure_cell_visible(GRID_SCROLLABLE_ID.clone(), self.active_cell)
                        .map(WorkpadMessage::ViewportChanged)
                }
                WorkpadMessage::PadNameEditStart => {
                    self.state = State::EditingPadName(self.pad.read().unwrap().name().to_owned());
                    Command::batch(vec![
                        text_input::focus(self.name_edit_id.clone()),
                        text_input::select_all(self.name_edit_id.clone()),
                    ])
                }
                WorkpadMessage::SheetNameEditStart => {
                    self.state = State::EditingSheetName(
                        self.pad.read().unwrap().active_sheet().name().to_owned(),
                    );
                    Command::batch(vec![
                        text_input::focus(self.sheet_edit_id.clone()),
                        text_input::select_all(self.sheet_edit_id.clone()),
                    ])
                }
                _ => Command::none(),
            },
            State::EditingPadName(new_name) => match message {
                WorkpadMessage::PadNameEdited(new_name) => {
                    self.state = State::EditingPadName(new_name);
                    Command::none()
                }
                WorkpadMessage::PadNameEditEnd => {
                    if !new_name.is_empty() {
                        self.pad.write().unwrap().set_name(new_name);
                        self.state = State::Passive;
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
            State::EditingSheetName(new_name) => match message {
                WorkpadMessage::SheetNameEdited(new_name) => {
                    self.state = State::EditingSheetName(new_name);
                    Command::none()
                }
                WorkpadMessage::SheetNameEditEnd => {
                    if !new_name.is_empty() {
                        //TODO self.pad.write().unwrap().set_name(new_name);
                        self.state = State::Passive;
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }
}
