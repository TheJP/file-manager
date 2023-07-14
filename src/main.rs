#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;

use std::{fs::File, path::Path};

use arguments::Arguments;
use clap::Parser;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image},
    App, Frame,
};
use egui_extras::RetainedImage;
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
    let images = load_images(&arguments.folder)?;

    eframe::run_native(
        "JP File Manager",
        Default::default(),
        Box::new(|_context| Box::new(FileManagerApp::new(images))),
    )?;

    Ok(())
}

fn load_images(folder_path: impl AsRef<Path>) -> Result<Vec<RetainedImage>> {
    use std::io::prelude::*;

    let mut images = Vec::new();

    for file in WalkDir::new(folder_path).sort_by_file_name() {
        let file = file?;
        let Ok(format) = ImageFormat::from_path(file.path()) else { continue; };

        if format.can_read() {
            println!("{:?}", file.file_name());
            let mut bytes = Vec::new();
            File::open(file.path())?.read_to_end(&mut bytes)?;

            images.push(
                RetainedImage::from_image_bytes(file.file_name().to_string_lossy(), &bytes)
                    .map_err(Error::DisplayImage)?,
            );
        }
    }

    Ok(images)
}

#[derive(Default)]
struct FileManagerApp {
    images: Vec<RetainedImage>,
    current_image: Option<usize>,
}

impl FileManagerApp {
    fn new(images: Vec<RetainedImage>) -> Self {
        let current_image = (images.len() > 0).then_some(0);
        Self {
            images,
            current_image,
        }
    }
}

impl App for FileManagerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Heading");

            if let Some(image_index) = self.current_image {
                let texture_id = self.images[image_index].texture_id(ctx);
                ui.add(Image::new(texture_id, vec2(200.0, 200.0)));
            }
        });
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not process given folder: {0}")]
    FindingFiles(#[from] walkdir::Error),

    #[error("could not load image file: {0}")]
    LoadImage(#[from] std::io::Error),

    #[error("could not decode image: {0}")]
    DisplayImage(String),

    #[error("UI: {0}")]
    UIError(#[from] eframe::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
