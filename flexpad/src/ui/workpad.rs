use std::{cell::RefCell, rc::Rc};

use crate::model::workpad::{Workpad, WorkpadMaster, WorkpadUpdate};
use flexpad_grid::{
    scroll::ensure_cell_visible, style, Border, Borders, CellRange, ColumnHead, Grid, GridCell,
    GridCorner, GridScrollable, RowCol, RowHead, SumSeq, Viewport,
};
use iced::{
    advanced::{mouse::click, widget},
    alignment, theme,
    widget::{button, column, horizontal_space, image, row, text, vertical_rule},
    Alignment, Color, Command, Element, Length, Subscription,
};
use iced_aw::{helpers::menu_bar, helpers::menu_tree, modal, ItemHeight, ItemWidth, MenuTree};

use self::{
    active::{ActiveCell, Editor},
    inactive::InactiveCell,
    pad_properties::{PadPropertiesMessage, PadPropertiesUi},
    sheet_properties::{SheetPropertiesMessage, SheetPropertiesUi},
};

use super::{images, SPACE_S};

mod active;
mod inactive;
mod pad_properties;
mod sheet_properties;

use once_cell::sync::Lazy;

static FORMULA_BAR_ID: Lazy<active::Id> = Lazy::new(active::Id::unique);
static ACTIVE_CELL_ID: Lazy<active::Id> = Lazy::new(active::Id::unique);

static GRID_SCROLLABLE_ID: Lazy<flexpad_grid::scroll::Id> =
    Lazy::new(flexpad_grid::scroll::Id::unique);

#[derive(Debug, Default)]
enum ShowModal {
    #[default]
    None,
    PadProperties(PadPropertiesUi),
    SheetProperties(SheetPropertiesUi),
}

impl ShowModal {
    fn into_pad_properties(self) -> PadPropertiesUi {
        match self {
            ShowModal::PadProperties(props) => props,
            _ => panic!("Expected PadProperties"),
        }
    }

    fn into_sheet_properties(self) -> SheetPropertiesUi {
        match self {
            ShowModal::SheetProperties(props) => props,
            _ => panic!("Expected SheetProperties"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkpadMessage {
    NoOp, // TODO Temporary
    Multi(Vec<WorkpadMessage>),
    PadClose,
    PadShowProperties,
    PadPropertiesMsg(PadPropertiesMessage),
    SheetShowProperties,
    SheetPropertiesMsg(SheetPropertiesMessage),
    SheetShowDetails,
    Focus(widget::Id),
    ViewportChanged(Viewport),
    ActiveCellMove(Move),
    ActiveCellNewValue(String),
}

impl std::fmt::Display for WorkpadMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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

pub struct WorkpadUI {
    pad_master: WorkpadMaster,
    pad: Workpad,
    visible_cells: CellRange,
    active_cell: RowCol,
    active_cell_editor: Rc<RefCell<active::Editor>>,
    focus: widget::Id,
    modal: ShowModal,
}

impl WorkpadUI {
    pub fn new(pad_master: WorkpadMaster) -> Self {
        let pad = pad_master.active_version();
        let active_cell_editor = {
            let active_sheet = pad.active_sheet();
            let active_cell = active_sheet.cell(0, 0);
            Editor::new(active_cell.value())
        };

        Self {
            pad_master,
            pad,
            visible_cells: CellRange::empty(),
            active_cell: (0, 0).into(),
            active_cell_editor: Rc::new(RefCell::new(active_cell_editor)),
            focus: ACTIVE_CELL_ID.clone().into(),
            modal: Default::default(),
        }
    }

    pub fn title(&self) -> String {
        self.pad.name().to_owned()
    }

    pub fn view(&self) -> iced::Element<'_, WorkpadMessage> {
        let screen = column![
            self.menu_bar(),
            self.toolbar_view(),
            self.sheet_and_formula_row_view(),
            self.grid_view(),
        ]
        .padding(10)
        .spacing(SPACE_S)
        .align_items(Alignment::Start)
        .into();

        match &self.modal {
            ShowModal::None => screen,
            ShowModal::PadProperties(ui) => modal(
                screen,
                Some(ui.view().map(WorkpadMessage::PadPropertiesMsg)),
            )
            .into(),
            ShowModal::SheetProperties(ui) => modal(
                screen,
                Some(ui.view().map(WorkpadMessage::SheetPropertiesMsg)),
            )
            .into(),
        }
    }

    // TODO Switch to system menus once available
    fn menu_bar(&self) -> iced::Element<'_, WorkpadMessage> {
        menu_bar(vec![
            menu_parent(
                "Workpad",
                vec![
                    menu_leaf("Properties...", WorkpadMessage::PadShowProperties),
                    // TODO No actual delete (since no actual save) at present
                    menu_leaf("Delete Workpad", WorkpadMessage::PadClose),
                    menu_leaf("Close Workpad", WorkpadMessage::PadClose),
                ],
            ),
            menu_parent(
                "Sheet",
                vec![
                    menu_leaf("Properties...", WorkpadMessage::SheetShowProperties),
                    menu_leaf("New Sheet", WorkpadMessage::NoOp),
                    menu_leaf("Delete Sheet", WorkpadMessage::NoOp),
                ],
            ),
        ])
        .item_width(ItemWidth::Uniform(180))
        .item_height(ItemHeight::Uniform(27))
        .into()
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
        .spacing(SPACE_S)
        .into()
    }

    fn sheet_and_formula_row_view(&self) -> iced::Element<'_, WorkpadMessage> {
        let button = |img, msg| {
            button(image(img))
                .on_press(msg)
                .width(Length::Shrink)
                .height(20)
                .padding(2)
                .style(theme::Button::Text)
        };

        let sheet: iced::Element<'_, WorkpadMessage> = row![
            text(self.pad.active_sheet().name()).size(14),
            horizontal_space(5),
            // TODO
            button(images::expand_more(), WorkpadMessage::SheetShowDetails),
        ]
        .width(200)
        .into();

        let active_sheet = self.pad.active_sheet();
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
        .spacing(SPACE_S)
        .into()
    }

    fn grid_view(&self) -> Element<'_, WorkpadMessage> {
        let sheet = self.pad.active_sheet();

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

        for rc in self.visible_cells.cells() {
            let RowCol {
                row: rw,
                column: cl,
            } = rc;
            if self.active_cell.row != rw || self.active_cell.column != cl {
                let cell = sheet.cell(rw as usize, cl as usize);
                let ic = InactiveCell::new(rc, cell.value())
                    // TODO Set details from spreadsheet data
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center)
                    .font_size(10.0);

                let grid_cell = GridCell::new((rw, cl), ic);
                grid = grid.push_cell(grid_cell);
            };
        }

        // Always add the active cell even when not visible so keystrokes are handled
        let ac = ActiveCell::new(self.active_cell_editor.clone())
            .id(ACTIVE_CELL_ID.clone())
            .focused(self.focus == ACTIVE_CELL_ID.clone().into())
            .edit_when_clicked(click::Kind::Double)
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

    pub(crate) fn subscription(&self) -> iced::Subscription<WorkpadMessage> {
        match &self.modal {
            ShowModal::None => Subscription::none(),
            ShowModal::PadProperties(props) => {
                props.subscription().map(WorkpadMessage::PadPropertiesMsg)
            }
            ShowModal::SheetProperties(props) => {
                props.subscription().map(WorkpadMessage::SheetPropertiesMsg)
            }
        }
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

        match message {
            WorkpadMessage::ViewportChanged(viewport) => {
                self.visible_cells = viewport.cell_range();
                Command::none()
            }
            WorkpadMessage::Focus(id) => {
                // TODO check for edit in progress?
                self.focus = id;
                Command::none()
            }
            WorkpadMessage::ActiveCellNewValue(s) => {
                // TODO Move this to model and perform updates (and recaclulations) on another thread
                let editor = Editor::new(&s);
                let update = self.pad.set_active_sheet_cell_value(
                    self.active_cell.row as usize,
                    self.active_cell.column as usize,
                    s,
                );
                self.update_pad(update);
                self.active_cell_editor = Rc::new(RefCell::new(editor));

                Command::none()
            }
            WorkpadMessage::ActiveCellMove(mve) => {
                let sheet = self.pad.active_sheet();
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
                    Move::To(rc) => {
                        self.active_cell = rc;
                    }
                }

                if prior_active_cell != self.active_cell {
                    let rw = self.active_cell.row as usize;
                    let cl = self.active_cell.column as usize;
                    let cell = sheet.cell(rw, cl);
                    let editor = Editor::new(cell.value());
                    let prior_editor = self.active_cell_editor.replace(editor);
                    self.focus = ACTIVE_CELL_ID.clone().into();

                    if prior_editor.is_editing() {
                        // TODO Move this to model and perform updates (and recaclulations) on another thread
                        let update = self.pad.set_active_sheet_cell_value(
                            prior_active_cell.row as usize,
                            prior_active_cell.column as usize,
                            prior_editor.contents(),
                        );
                        self.update_pad(update);
                    }
                }

                ensure_cell_visible(GRID_SCROLLABLE_ID.clone(), self.active_cell)
                    .map(WorkpadMessage::ViewportChanged)
            }
            WorkpadMessage::PadShowProperties => {
                self.modal = ShowModal::PadProperties(PadPropertiesUi::new(self.pad.clone()));
                Command::none()
            }
            WorkpadMessage::PadPropertiesMsg(PadPropertiesMessage::Finish(ok_cancel)) => {
                let mut modal = ShowModal::None;
                std::mem::swap(&mut modal, &mut self.modal);
                if ok_cancel.is_ok() {
                    self.update_pad(modal.into_pad_properties().into_update())
                }
                Command::none()
            }
            WorkpadMessage::PadPropertiesMsg(msg) => {
                if let ShowModal::PadProperties(ref mut props) = self.modal {
                    props.update(msg).map(WorkpadMessage::PadPropertiesMsg)
                } else {
                    panic!("PadPropertiesMsg not expected for this modal")
                }
            }
            WorkpadMessage::SheetShowProperties => {
                self.modal =
                    ShowModal::SheetProperties(SheetPropertiesUi::new(self.pad.active_sheet()));
                Command::none()
            }
            WorkpadMessage::SheetPropertiesMsg(SheetPropertiesMessage::Finish(ok_cancel)) => {
                let mut modal = ShowModal::None;
                std::mem::swap(&mut modal, &mut self.modal);
                if ok_cancel.is_ok() {
                    self.update_pad(modal.into_sheet_properties().into_update())
                }
                Command::none()
            }
            WorkpadMessage::SheetPropertiesMsg(msg) => {
                if let ShowModal::SheetProperties(ref mut props) = self.modal {
                    props.update(msg).map(WorkpadMessage::SheetPropertiesMsg)
                } else {
                    panic!("PadPropertiesMsg not expected for this modal")
                }
            }
            WorkpadMessage::SheetShowDetails => {
                dbg!("Show sheet details");
                Command::none()
            }
            WorkpadMessage::PadClose => Command::none(),
            WorkpadMessage::Multi(_) => Command::none(),
            WorkpadMessage::NoOp => Command::none(),
        }
    }

    pub fn update_pad(&mut self, update: WorkpadUpdate) {
        self.pad_master.update(update);
        self.pad = self.pad_master.active_version();
    }
}

fn menu_parent<'a>(
    label: &str,
    children: Vec<MenuTree<'a, WorkpadMessage, iced::Renderer>>,
) -> MenuTree<'a, WorkpadMessage, iced::Renderer> {
    menu_tree(
        button(
            text(label)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(alignment::Vertical::Center),
        )
        .padding([4, 8])
        .style(iced::theme::Button::Custom(Box::new(
            MenuLeafButtonStyle {},
        )))
        // op_press to stop item appearing disabled
        .on_press(WorkpadMessage::NoOp),
        children,
    )
}

fn menu_leaf(label: &str, msg: WorkpadMessage) -> MenuTree<'_, WorkpadMessage, iced::Renderer> {
    let none: Vec<iced_aw::menu::menu_tree::MenuTree<'_, WorkpadMessage>> = vec![];
    menu_tree(
        button(
            text(label)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(alignment::Vertical::Center),
        )
        .padding([4, 8])
        .style(iced::theme::Button::Custom(Box::new(
            MenuLeafButtonStyle {},
        )))
        .on_press(msg),
        none,
    )
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
