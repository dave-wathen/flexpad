use crate::version::Version;
use egui::{Align, Layout, Separator, Vec2};

use crate::images::AppImages;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FlexpadApp {
    #[serde(skip)]
    images: AppImages,
    #[serde(skip)]
    version: Version,
    #[serde(skip)]
    state: UiState,
}

enum UiState {
    FrontScreen,
    Workpad,
}

impl Default for FlexpadApp {
    fn default() -> Self {
        Self {
            images: Default::default(),
            version: Default::default(),
            state: UiState::FrontScreen,
        }
    }
}

impl FlexpadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn render_front_page(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_margin")
            .show_separator_line(false)
            .show(ctx, |ui| ui.add_space(20.0));
        egui::SidePanel::right("right_margin")
            .show_separator_line(false)
            .show(ctx, |ui| ui.add_space(20.0));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add(self.images.app_icon.image(ctx));
                ui.label(self.version.description());
                ui.add(Separator::default().spacing(20.0));
                ui.vertical(|ui| {
                    ui.heading("Create New ...");
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(80.0, 40.0),
                            Layout::top_down(Align::Center),
                            |ui| {
                                if ui.add(self.images.workpad_icon.image_button(ctx)).clicked() {
                                    self.state = UiState::Workpad
                                }
                                ui.label("Workpad")
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
    }

    fn render_workpad(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Pad", |ui| {
                    if ui.button("Close").clicked() {
                        ui.close_menu();
                        self.state = UiState::FrontScreen;
                    }

                    // Can only quit native windows
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        ui.close_menu();
                        _frame.close();
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Unnamed");
            })
        });
    }
}

impl eframe::App for FlexpadApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.state {
            UiState::FrontScreen => self.render_front_page(ctx),
            UiState::Workpad => self.render_workpad(ctx, frame),
        };
    }
}
