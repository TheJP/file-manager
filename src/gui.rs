use super::load_image;
use eframe::Frame;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image, Key, Ui},
    App,
};
use egui_extras::{RetainedImage, Size, StripBuilder};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct FileManagerApp {
    image_paths: Vec<(usize, PathBuf)>,
    current_image: Option<usize>,
    image_cache: HashMap<usize, RetainedImage>,
}

impl FileManagerApp {
    pub(crate) fn new(image_paths: Vec<(usize, PathBuf)>) -> Self {
        let current_image = (!image_paths.is_empty()).then_some(0);
        Self {
            image_paths,
            current_image,
            ..Default::default()
        }
    }

    pub(crate) fn add_image(&mut self, ctx: &Context, ui: &mut Ui) {
        let Some(image_index) = self.current_image else {
            return;
        };
        let (key, path) = &self.image_paths[image_index];

        if !self.image_cache.contains_key(key) {
            match load_image(path) {
                Ok(image) => {
                    self.image_cache.insert(*key, image);
                }
                Err(error) => {
                    eprintln!(
                        "Encountered error while loading image {:?}: {}",
                        path.file_name().unwrap_or_default(),
                        error
                    );
                    return;
                }
            }
        }

        let image = &self.image_cache[key];

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

    pub(crate) fn navigate_images(&mut self, ctx: &Context) {
        let Some(change) = &mut self.current_image else {
            return;
        };

        if ctx.input(|input| input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::PageDown))
        {
            *change = if *change + 1 >= self.image_paths.len() {
                0
            } else {
                *change + 1
            };
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::PageUp)) {
            *change = if *change > 0 {
                *change - 1
            } else {
                self.image_paths.len() - 1
            };
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
