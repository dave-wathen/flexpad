mod front_screen;
mod workpad_screen;

use crate::version::Version;

use workpad_screen::WorkpadScreen;

use self::front_screen::FrontScreen;

use super::images::AppImages;

enum UiState {
    FrontScreen(FrontScreen),
    Workpad(WorkpadScreen),
}

pub enum AppAction {
    OpenNewWorkpad,
    CloseWorkpad,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

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

impl Default for FlexpadApp {
    fn default() -> Self {
        Self {
            images: Default::default(),
            version: Default::default(),
            state: UiState::FrontScreen(Default::default()),
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
}

impl eframe::App for FlexpadApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let actions = match self.state {
            UiState::FrontScreen(ref mut screen) => {
                screen.render(&mut self.images, &self.version, ctx)
            }
            UiState::Workpad(ref mut screen) => screen.render(ctx),
        };

        for action in actions {
            match action {
                AppAction::OpenNewWorkpad => self.state = UiState::Workpad(Default::default()),
                AppAction::CloseWorkpad => self.state = UiState::FrontScreen(Default::default()),
                #[cfg(not(target_arch = "wasm32"))]
                AppAction::Quit => _frame.close(),
            }
        }
    }
}
