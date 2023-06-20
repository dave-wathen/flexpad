use std::{ops::RangeInclusive, sync::Arc};

use egui::{
    Align2, Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, ScrollArea, Sense, Stroke,
    Vec2, Widget,
};

use crate::model::workpad::{Sheet, Workpad};

pub struct WorkpadUiState {
    pad: Arc<Workpad>,
    edit: WorkpadEdit,
}

impl Default for WorkpadUiState {
    fn default() -> Self {
        Self {
            pad: Arc::new(Default::default()),
            edit: Default::default(),
        }
    }
}

#[derive(Debug)]
enum WorkpadEdit {
    None,
    SheetName(String),
    Formula(String),
}

impl Default for WorkpadEdit {
    fn default() -> Self {
        WorkpadEdit::None
    }
}

pub struct WorkpadUi<'a> {
    state: &'a mut WorkpadUiState,
    visible_rows: RangeInclusive<usize>,
    visible_columns: RangeInclusive<usize>,
    gridline: Stroke,
}

const EMPTY: RangeInclusive<usize> = RangeInclusive::new(1, 0);

impl WorkpadUi<'_> {
    pub fn new(state: &mut WorkpadUiState) -> WorkpadUi<'_> {
        WorkpadUi {
            state,
            visible_rows: EMPTY.clone(),
            visible_columns: EMPTY.clone(),
            gridline: Stroke::new(1.0, Color32::LIGHT_GRAY),
        }
    }

    fn current_sheet(&self) -> Sheet<'_> {
        self.state.pad.current_sheet()
    }

    fn start_edit(&mut self, edit: WorkpadEdit) {
        self.finish_edit();
        self.state.edit = edit;
    }

    fn finish_edit(&mut self) {
        if let WorkpadEdit::None = self.state.edit {
        } else {
            println!("Send {:?}", self.state.edit);
            self.state.edit = WorkpadEdit::None;
        };
    }

    fn cancel_edit(&mut self) {
        self.state.edit = WorkpadEdit::None;
    }

    fn calc_visible_cells(&mut self, viewport: Rect) {
        // TODO probably more efficient to maintain vec of cumulative heights
        // Alternatively try_fold with ControlFlow can eliminate iterating the tail
        let (_, found, lo, hi) = self.current_sheet().rows().fold(
            (0.0, false, 0, 0),
            |(height, lo_found, lo, hi), row| {
                if !lo_found {
                    let mid_y = height + row.height() / 2.0;
                    if viewport.y_range().contains(&mid_y) {
                        (row.height(), true, row.index(), row.index())
                    } else {
                        (height + row.height(), false, 0, 0)
                    }
                } else if height < viewport.height() {
                    (height + row.height(), true, lo, row.index())
                } else {
                    (height, true, lo, hi)
                }
            },
        );
        self.visible_rows = if found { lo..=hi } else { EMPTY.clone() };

        // TODO probably more efficient to maintain vec of cumulative widths
        // Alternatively try_fold with ControlFlow can eliminate iterating the tail
        let (_, found, lo, hi) = self.current_sheet().columns().fold(
            (0.0, false, 0, 0),
            |(width, lo_found, lo, hi), column| {
                if !lo_found {
                    let mid_x = width + column.width() / 2.0;
                    if viewport.x_range().contains(&mid_x) {
                        (column.width(), true, column.index(), column.index())
                    } else {
                        (width + column.width(), false, 0, 0)
                    }
                } else if width < viewport.width() {
                    (width + column.width(), true, lo, column.index())
                } else {
                    (width, true, lo, hi)
                }
            },
        );
        self.visible_columns = if found { lo..=hi } else { EMPTY.clone() };
    }

    fn render_corner(&self, ui: &mut egui::Ui) -> Rect {
        let sheet = self.state.pad.current_sheet();
        let available_rect = ui.available_rect_before_wrap();
        let visuals = ui.style().visuals.clone();

        let corner_rect = Rect::from_min_size(
            available_rect.min,
            Vec2::new(sheet.row_header_width(), sheet.column_header_height()),
        );

        ui.painter()
            .rect_filled(corner_rect, Rounding::none(), self.gridline.color);
        ui.painter().rect_filled(
            corner_rect.shrink(2.0),
            Rounding::none(),
            visuals.window_fill,
        );

        corner_rect
    }

    fn render_grid(&mut self, ui: &mut egui::Ui) {
        let pad = self.state.pad.clone();
        let sheet = pad.current_sheet();

        ScrollArea::both()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, viewport| {
                let height = sheet.rows().map(|r| r.height()).sum();
                let width = sheet.columns().map(|c| c.width()).sum();
                ui.set_height(height);
                ui.set_width(width);

                let mut used_rect = Rect::NOTHING;

                self.calc_visible_cells(viewport);
                if !self.visible_rows.is_empty() && !self.visible_columns.is_empty() {
                    let mut y = ui.min_rect().top() + viewport.top();

                    for rw in self.visible_rows.clone() {
                        let row = sheet.row(rw);
                        let height = row.height();

                        let mut x = ui.min_rect().left() + viewport.left();

                        for cl in self.visible_columns.clone() {
                            let cell = sheet.cell(rw, cl);
                            let reference = cell.name();
                            let cell_rect = Rect::from_two_pos(
                                Pos2::new(x, y),
                                Pos2::new(x + cell.width(), y + height),
                            );
                            used_rect = used_rect.union(cell_rect);

                            ui.painter()
                                .rect_stroke(cell_rect, Rounding::none(), self.gridline);
                            ui.painter().text(
                                cell_rect.center(),
                                Align2::CENTER_CENTER,
                                reference,
                                FontId {
                                    size: 10.0,
                                    family: FontFamily::Proportional,
                                },
                                ui.style().visuals.warn_fg_color,
                            );
                            x += cell.width();
                        }
                        y += height;
                    }
                }
            });
    }

    fn render_row_headings(&mut self, ui: &mut egui::Ui) {
        let sheet = self.state.pad.current_sheet();

        let mut y = ui.min_rect().top();
        let x_range = ui.max_rect().x_range();
        for rw in self.visible_rows.clone() {
            let row = sheet.row(rw);

            let height = row.height();
            let rect = Rect::from_x_y_ranges(x_range.clone(), y..=(y + height));

            ui.painter()
                .rect_stroke(rect, Rounding::none(), self.gridline);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                row.name(),
                FontId {
                    size: 10.0,
                    family: FontFamily::Proportional,
                },
                ui.style().visuals.text_color(),
            );
            y += height;
        }
    }

    fn render_column_headings(&mut self, ui: &mut egui::Ui) {
        let sheet = self.state.pad.current_sheet();

        let mut x = ui.min_rect().left();
        let y_range = ui.max_rect().y_range();
        for col in self.visible_columns.clone() {
            let column = sheet.column(col);

            let width = column.width();
            let rect = Rect::from_x_y_ranges(x..=(x + width), y_range.clone());

            ui.painter()
                .rect_stroke(rect, Rounding::none(), self.gridline);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                column.name(),
                FontId {
                    size: 10.0,
                    family: FontFamily::Proportional,
                },
                ui.style().visuals.text_color(),
            );
            x += width;
        }
    }
}

impl Widget for WorkpadUi<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Unnamed");
            ui.horizontal(|ui| {
                let _ = ui.button("a");
                let _ = ui.button("b");
                let _ = ui.button("c");
                let _ = ui.button("d");
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.push_id("sheet_name", |ui| {
                    if let WorkpadEdit::SheetName(ref mut s) = self.state.edit {
                        let resp = ui.text_edit_singleline(s);
                        if resp.lost_focus() {
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                self.cancel_edit();
                            } else {
                                self.finish_edit();
                            }
                        }
                    } else {
                        let mut sheet_name = self.current_sheet().name().to_owned();
                        let resp = ui.text_edit_singleline(&mut sheet_name);
                        if resp.gained_focus() {
                            self.finish_edit();
                        }
                        if resp.changed() {
                            self.start_edit(WorkpadEdit::SheetName(sheet_name));
                        };
                    };
                });
                ui.separator();
                ui.push_id("formula", |ui| {
                    if let WorkpadEdit::Formula(ref mut s) = self.state.edit {
                        let resp = ui.text_edit_singleline(s);
                        if resp.lost_focus() {
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                self.cancel_edit();
                            } else {
                                self.finish_edit();
                            }
                        }
                    } else {
                        let mut formula = "Formula".to_owned();
                        let resp = ui.text_edit_singleline(&mut formula);
                        if resp.gained_focus() {
                            self.finish_edit();
                        }
                        if resp.changed() {
                            self.start_edit(WorkpadEdit::Formula(formula));
                        };
                    }
                });
            });

            let corner_rect = self.render_corner(ui);
            let available_rect = ui.available_rect_before_wrap();
            let grid_rect = Rect::from_min_size(
                corner_rect.right_bottom(),
                available_rect.size() - corner_rect.size(),
            );
            let row_headings_rect = Rect::from_min_size(
                corner_rect.left_bottom(),
                Vec2::new(corner_rect.width(), grid_rect.height()),
            );
            let column_headings_rect = Rect::from_min_size(
                corner_rect.right_top(),
                Vec2::new(grid_rect.width(), corner_rect.height()),
            );

            let mut content_ui = ui.child_ui(grid_rect, *ui.layout());
            self.render_grid(&mut content_ui);

            let mut row_headings_ui = ui.child_ui(row_headings_rect, *ui.layout());
            self.render_row_headings(&mut row_headings_ui);

            let mut column_headings_ui = ui.child_ui(column_headings_rect, *ui.layout());
            self.render_column_headings(&mut column_headings_ui);

            ui.allocate_rect(available_rect, Sense::hover());
        })
        .response
    }
}
