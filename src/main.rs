#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use arguments::Arguments;
use clap::Parser;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image, Key, Ui},
    App, Frame,
};
use egui_extras::{RetainedImage, Size, StripBuilder};
use image::ImageFormat;
use thiserror::Error;
use walkdir::WalkDir;

fn main() {
    let arguments = Arguments::parse();
    if let Err(error) = file_manager(&arguments) {
        eprintln!("Encountered error in file manager: {}", error);
    }
}

fn file_manager(arguments: &Arguments) -> Result<()> {
    let image_paths = find_images(&arguments.folder)?;

    eframe::run_native(
        "JP File Manager",
        Default::default(),
        Box::new(|_context| Box::new(FileManagerApp::new(image_paths))),
    )?;

    Ok(())
}

fn find_images(folder_path: impl AsRef<Path>) -> Result<Vec<(usize, PathBuf)>> {
    let mut images = Vec::new();
    let mut index = 0;

    for file in WalkDir::new(folder_path).sort_by_file_name() {
        let file = file?;
        let Ok(format) = ImageFormat::from_path(file.path()) else { continue; };
        if format.can_read() {
            images.push((index, file.into_path()));
            index += 1;
        }
    }

    Ok(images)
}

fn load_image(path: impl AsRef<Path>) -> Result<RetainedImage> {
    use std::io::prelude::*;
    let mut bytes = Vec::new();
    File::open(&path)?.read_to_end(&mut bytes)?;

    let debug_name = path
        .as_ref()
        .file_name()
        .map_or("[Image]".into(), |name| name.to_string_lossy());

    RetainedImage::from_image_bytes(debug_name, &bytes).map_err(Error::DisplayImage)
}

#[derive(Default)]
struct FileManagerApp {
    image_paths: Vec<(usize, PathBuf)>,
    current_image: Option<usize>,
    image_cache: HashMap<usize, RetainedImage>,
}

impl FileManagerApp {
    fn new(image_paths: Vec<(usize, PathBuf)>) -> Self {
        let current_image = (!image_paths.is_empty()).then_some(0);
        Self {
            image_paths,
            current_image,
            ..Default::default()
        }
    }

    fn add_image(&mut self, ctx: &Context, ui: &mut Ui) {
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

    fn navigate_images(&mut self, ctx: &Context) {
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not process given folder: {0}")]
    FindingFiles(#[from] walkdir::Error),

    #[error("could not read image file: {0}")]
    LoadImage(#[from] std::io::Error),

    #[error("could not decode image: {0}")]
    DisplayImage(String),

    #[error("UI: {0}")]
    UIError(#[from] eframe::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
