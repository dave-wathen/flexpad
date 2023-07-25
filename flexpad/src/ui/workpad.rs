use std::sync::{Arc, RwLock};

use crate::model::workpad::Workpad;
use flexpad_grid::{
    style, CellRange, ColumnHead, Grid, GridCell, GridCorner, GridScrollable, RowCol, RowHead,
    SumSeq, Viewport,
};
use iced::{
    alignment, theme,
    widget::{self, button, column, horizontal_space, image, row, text, text_input, vertical_rule},
    Alignment, Command, Element, Length,
};

use super::images;

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
    PadClose,
    PadNameEditStart,
    PadNameEdited(String),
    PadNameEditEnd,
    SheetNameEditStart,
    SheetNameEdited(String),
    SheetNameEditEnd,
    ViewportChanged(Viewport),
}

pub struct WorkpadUI {
    pad: Arc<RwLock<Workpad>>,
    state: State,
    name_edit_id: text_input::Id,
    sheet_edit_id: text_input::Id,
    visible_cells: Option<CellRange>,
}

impl Default for WorkpadUI {
    fn default() -> Self {
        Self {
            pad: Default::default(),
            state: Default::default(),
            name_edit_id: text_input::Id::unique(),
            sheet_edit_id: text_input::Id::unique(),
            visible_cells: None,
        }
    }
}

impl WorkpadUI {
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
            button(widget::image(img))
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
            button(widget::image(img))
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
            button(widget::image(img))
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

        let cell_name: iced::Element<'_, WorkpadMessage> =
            text(self.pad.read().unwrap().active_sheet().active_cell().name())
                .size(18)
                .width(100)
                .horizontal_alignment(alignment::Horizontal::Center)
                .into();

        let formula: iced::Element<'_, WorkpadMessage> = text("formula")
            .size(18)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Left)
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

        if let Some(visibles) = self.visible_cells {
            for cl in visibles.columns() {
                let column = sheet.column(cl as usize);
                grid = grid.push_column_head(ColumnHead::new(cl, text(column.name()).size(10)))
            }

            for rw in visibles.rows() {
                let row = sheet.row(rw as usize);
                grid = grid.push_row_head(RowHead::new(rw, text(row.name()).size(10)))
            }

            for RowCol {
                row: rw,
                column: cl,
            } in visibles.cells()
            {
                let cell = sheet.cell(rw as usize, cl as usize);
                grid = grid.push_cell(GridCell::new((rw, cl), text(cell.name()).size(10)))
            }
        }

        GridScrollable::new(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_viewport_change(WorkpadMessage::ViewportChanged)
            .into()
    }

    // TODO Cancel editing using Esc/Button
    pub fn update(&mut self, message: WorkpadMessage) -> Command<WorkpadMessage> {
        if let WorkpadMessage::ViewportChanged(viewport) = message {
            self.visible_cells = Some(viewport.cell_range())
        }

        match &self.state {
            State::Passive => match message {
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
