use crate::images::ImageCache;

use eframe::Frame;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image, Key, Ui},
    App,
};
use egui_extras::{Size, StripBuilder};

#[derive(Default)]
pub(crate) struct FileManagerApp {
    images: ImageCache,
}

impl FileManagerApp {
    pub(crate) fn new(images: ImageCache) -> Self {
        Self { images }
    }

    fn add_image(&mut self, ctx: &Context, ui: &mut Ui) {
        let Some(image) = self.images.current_image().transpose() else {
            return;
        };

        let Ok(image) = image else {
            eprintln!("Encountered error while loading image: {}", image.err().unwrap());
            return;
        };

        let size = ui.available_size_before_wrap().max(vec2(100.0, 100.0));
        let aspect_x = size.y * (image.width() as f32) / (image.height() as f32);
        let fit = (aspect_x <= size.x)
            .then_some(vec2(aspect_x, size.y))
            .unwrap_or_else(|| {
                vec2(
                    size.x,
                    size.x * (image.height() as f32) / (image.width() as f32),
                )
            });

        ui.vertical_centered(|ui| ui.add(Image::new(image.texture_id(ctx), fit)));
    }

    fn navigate_images(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::PageDown))
        {
            self.images.forward();
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::PageUp)) {
            self.images.back();
        }
    }
}

impl App for FileManagerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.navigate_images(ctx);

        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| self.add_image(ctx, ui));
                });
        });
    }
}
