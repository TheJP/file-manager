#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;
mod gui;
mod images;

use arguments::Arguments;
use clap::Parser;

use thiserror::Error;

fn main() {
    let arguments = Arguments::parse();
    if let Err(error) = file_manager(&arguments) {
        eprintln!("Encountered error in file manager: {}", error);
    }
}

fn file_manager(arguments: &Arguments) -> Result<()> {
    let images = images::find(&arguments.folder)?;

    let app = Box::new(gui::FileManagerApp::new(images));
    eframe::run_native(
        "JP File Manager",
        Default::default(),
        Box::new(|_context| app),
    )?;

    Ok(())
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
