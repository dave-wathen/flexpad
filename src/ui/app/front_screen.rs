use egui::{Align, Layout, Separator, Vec2};

use crate::{ui::images::AppImages, version::Version};

use super::AppAction;

#[derive(Default)]
pub struct FrontScreen;

impl FrontScreen {
    pub fn render(
        &mut self,
        images: &mut AppImages,
        version: &Version,
        ctx: &egui::Context,
    ) -> Vec<AppAction> {
        let mut result = vec![];

        egui::SidePanel::left("left_margin")
            .show_separator_line(false)
            .show(ctx, |ui| ui.add_space(20.0));
        egui::SidePanel::right("right_margin")
            .show_separator_line(false)
            .show(ctx, |ui| ui.add_space(20.0));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add(images.app_icon.image(ctx));
                ui.label(version.description());
                ui.add(Separator::default().spacing(20.0));
                ui.vertical(|ui| {
                    ui.heading("Create New ...");
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(80.0, 40.0),
                            Layout::top_down(Align::Center),
                            |ui| {
                                if ui.add(images.workpad_icon.image_button(ctx)).clicked() {
                                    result.push(AppAction::OpenNewWorkpad);
                                }
                                ui.label("Workpad");
                            },
                        )
                    })
                });
                ui.add(Separator::default().spacing(20.0));
                ui.vertical(|ui| {
                    ui.heading("Reopen ...");
                });
            })
        });

        result
    }
}
