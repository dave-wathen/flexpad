use egui::{ColorImage, TextureHandle};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FlexpadApp {
    #[serde(skip)]
    icon: Option<TextureHandle>,
    #[serde(skip)]
    version: &'static str,
    #[serde(skip)]
    git_branch: &'static str,
    #[serde(skip)]
    git_sha: &'static str,
}

impl Default for FlexpadApp {
    fn default() -> Self {
        Self {
            icon: None,
            version: "unknown",
            git_branch: "unknown",
            git_sha: "unknown",
        }
    }
}

impl FlexpadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let mut result: FlexpadApp;
        if let Some(storage) = cc.storage {
            result = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            result = Default::default();
        };

        result.version = env!("CARGO_PKG_VERSION");
        result.git_branch = env!("VERGEN_GIT_BRANCH");
        result.git_sha = env!("VERGEN_GIT_SHA");
        result
    }

    fn render_front_page(&mut self, ctx: &egui::Context) {
        let product = if self.git_branch == self.version {
            format!("Flexpad {} ({})", self.version, &self.git_sha[0..7])
        } else {
            format!(
                "Flexpad {} ({} {})",
                self.version,
                self.git_branch,
                &self.git_sha[0..7]
            )
        };

        let icon: &TextureHandle = self.icon.get_or_insert_with(|| {
            load_image_from_memory(
                include_bytes!("../resources/FlexpadIcon.jpg"),
                ctx,
                "FlexpadIcon.jpg",
            )
            .expect("FlexpadIcon.jpg not initialized")
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.image(icon, icon.size_vec2());
                ui.label(product);
            })
        });
    }
}

impl eframe::App for FlexpadApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_front_page(ctx);
    }
}

fn load_image_from_memory(
    image_data: &[u8],
    ctx: &egui::Context,
    debug_name: &'static str,
) -> Result<TextureHandle, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    let texture = ctx.load_texture(debug_name, color_image, Default::default());
    Ok(texture)
}
