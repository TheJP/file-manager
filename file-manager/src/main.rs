#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;
mod gui;
mod images;

use arguments::Arguments;
use clap::Parser;

use eframe::egui::Style;
use rayon::ThreadPoolBuildError;
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
        Box::new(|creation_context| {
            apply_style(creation_context);
            app
        }),
    )?;

    Ok(())
}

fn apply_style(creation_context: &eframe::CreationContext<'_>) {
    let ctx = &creation_context.egui_ctx;
    let mut style = Style::clone(&ctx.style());
    style.visuals.window_shadow.extrusion = 0.0;
    ctx.set_style(style);
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

    #[error("could not create thread pool: {0}")]
    RayonError(#[from] ThreadPoolBuildError),

    #[error("{0}")]
    ChannelError(#[from] images::ChannelError),
}
pub type Result<T> = std::result::Result<T, Error>;
