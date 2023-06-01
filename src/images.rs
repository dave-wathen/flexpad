use egui::{ColorImage, Context, Image, ImageButton, TextureHandle};

pub struct AppImages {
    pub app_icon: AppImage,
    pub workpad_icon: AppImage,
}

pub struct AppImage {
    texture: Option<TextureHandle>,
    bytes: &'static [u8],
    debug_name: &'static str,
}

impl AppImage {
    fn new(bytes: &'static [u8], debug_name: &'static str) -> Self {
        Self {
            texture: None,
            bytes,
            debug_name,
        }
    }

    pub fn image(&mut self, ctx: &Context) -> Image {
        let icon: &TextureHandle = self.texture(ctx);
        Image::new(icon.id(), icon.size_vec2())
    }

    pub fn image_button(&mut self, ctx: &Context) -> ImageButton {
        let icon: &TextureHandle = self.texture(ctx);
        ImageButton::new(icon, icon.size_vec2())
    }

    fn texture(&mut self, ctx: &Context) -> &TextureHandle {
        self.texture
            .get_or_insert_with(|| match image::load_from_memory(self.bytes) {
                Ok(image) => {
                    let size = [image.width() as _, image.height() as _];
                    let image_buffer = image.to_rgba8();
                    let pixels = image_buffer.as_flat_samples();
                    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    ctx.load_texture(self.debug_name, color_image, Default::default())
                }
                Err(_) => panic!("{} not initialized", self.debug_name),
            })
    }
}

impl Default for AppImages {
    fn default() -> Self {
        Self {
            app_icon: AppImage::new(
                include_bytes!("../resources/FlexpadIcon.jpg"),
                "FlexpadIcon.jpg",
            ),
            workpad_icon: AppImage::new(include_bytes!("../resources/Workpad.png"), "Workpad.png"),
        }
    }
}
