#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;
mod image_cache;

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use arguments::Arguments;
use clap::Parser;
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
    let image_paths = find_images(&arguments.folder)?;

    eframe::run_native(
        "JP File Manager",
        Default::default(),
        Box::new(|_context| Box::new(gui::FileManagerApp::new(image_paths))),
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

mod gui;

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
