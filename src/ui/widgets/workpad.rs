use std::{ops::RangeInclusive, sync::Arc};

use egui::{
    Align2, Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, ScrollArea, Sense, Stroke,
    Vec2, Widget,
};

use crate::model::workpad::Workpad;

pub struct WorkpadUi {
    pad: Arc<Workpad>,
    visible_rows: RangeInclusive<usize>,
    visible_columns: RangeInclusive<usize>,
    gridline: Stroke,
}

const EMPTY: RangeInclusive<usize> = RangeInclusive::new(1, 0);

impl WorkpadUi {
    pub fn new(pad: Arc<Workpad>) -> Self {
        Self {
            pad,
            visible_rows: EMPTY.clone(),
            visible_columns: EMPTY.clone(),
            gridline: Stroke::new(1.0, Color32::LIGHT_GRAY),
        }
    }

    fn calc_visible_cells(&mut self, viewport: Rect) {
        let rows = self.pad.row_count();
        let columns = self.pad.column_count();

        // Find the first visible row
        let mut rows_height = 0.0;
        let mut visible_rows = EMPTY.clone();
        for rw in 0..rows {
            let mid_y = rows_height + self.pad.row_height(rw) / 2.0;
            rows_height += self.pad.row_height(rw);

            if viewport.y_range().contains(&mid_y) {
                visible_rows = rw..=rw;
                break;
            }
        }

        // Extend the range for further visible rows
        if !visible_rows.is_empty() {
            let mut visible_rows_height = 0.0;
            let mut rw = *visible_rows.start();
            while visible_rows_height < viewport.height() && rw < rows {
                visible_rows = *visible_rows.start()..=rw;
                visible_rows_height += self.pad.row_height(rw);
                rw += 1;
            }
        }

        // Find the first visible column
        let mut columns_width = 0.0;
        let mut visible_columns = EMPTY.clone();
        for cl in 0..columns {
            let mid_x = columns_width + self.pad.column_width(cl) / 2.0;
            columns_width += self.pad.column_width(cl);

            if viewport.x_range().contains(&mid_x) {
                visible_columns = cl..=cl;
                break;
            }
        }

        // Extend the range for further visible columns
        if !visible_columns.is_empty() {
            let mut visible_columns_width = 0.0;
            let mut cl = *visible_columns.start();
            while visible_columns_width < viewport.width() && cl < columns {
                visible_columns = *visible_columns.start()..=cl;
                visible_columns_width += self.pad.column_width(cl);
                cl += 1;
            }
        }

        self.visible_rows = visible_rows;
        self.visible_columns = visible_columns;
    }

    fn render_corner(&self, ui: &mut egui::Ui) -> Rect {
        let available_rect = ui.available_rect_before_wrap();
        let visuals = ui.style().visuals.clone();

        let corner_rect = Rect::from_min_size(
            available_rect.min,
            Vec2::new(self.pad.row_header_width(), self.pad.column_header_height()),
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
        ScrollArea::both()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, viewport| {
                let rows = self.pad.row_count();
                let columns = self.pad.column_count();

                let mut height = 0.0;
                for rw in 0..rows {
                    height += self.pad.row_height(rw);
                }
                ui.set_height(height);

                let mut width = 0.0;
                for cl in 0..columns {
                    width += self.pad.column_width(cl);
                }
                ui.set_width(width);

                let mut used_rect = Rect::NOTHING;

                self.calc_visible_cells(viewport);
                if !self.visible_rows.is_empty() && !self.visible_columns.is_empty() {
                    let mut y = ui.min_rect().top() + viewport.top();

                    for row in self.visible_rows.clone() {
                        let height = self.pad.row_height(row);

                        let mut x = ui.min_rect().left() + viewport.left();

                        for col in self.visible_columns.clone() {
                            let _cell = self.pad.cell(row, col);
                            let reference = &_cell.a1_reference() as &str;
                            let width = self.pad.column_width(col);
                            let cell_rect = Rect::from_two_pos(
                                Pos2::new(x, y),
                                Pos2::new(x + width, y + height),
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
                            x += width;
                        }
                        y += height;
                    }
                }
            });
    }

    fn render_row_headings(&mut self, ui: &mut egui::Ui) {
        let mut y = ui.min_rect().top();
        let x_range = ui.max_rect().x_range();
        for row in self.visible_rows.clone() {
            let height = self.pad.row_height(row);
            let rect = Rect::from_x_y_ranges(x_range.clone(), y..=(y + height));

            ui.painter()
                .rect_stroke(rect, Rounding::none(), self.gridline);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                (row + 1).to_string(),
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
        let mut x = ui.min_rect().left();
        let y_range = ui.max_rect().y_range();
        for col in self.visible_columns.clone() {
            let width = self.pad.column_width(col);
            let rect = Rect::from_x_y_ranges(x..=(x + width), y_range.clone());

            ui.painter()
                .rect_stroke(rect, Rounding::none(), self.gridline);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                self.pad.column(col).name(),
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

impl Widget for WorkpadUi {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
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

        ui.allocate_rect(available_rect, Sense::hover())
    }
}
