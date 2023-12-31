#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod arguments;
mod gui;
mod images;

use std::path::{Path, PathBuf};

use arguments::Arguments;

use clap::Parser;
use directories::ProjectDirs;
use eframe::egui::Style;
use rayon::ThreadPoolBuildError;
use thiserror::Error;

const REPOSITORY_PATH: &str = ".meta";

fn main() {
    let arguments = Arguments::parse();
    if let Err(error) = file_manager(&arguments) {
        eprintln!("Encountered error in file manager: {}", error);
    }
}

fn file_manager(arguments: &Arguments) -> Result<()> {
    let folder_path = &arguments.folder.canonicalize()?;
    let images = images::find(folder_path)?;
    let mut meta = meta::Repository::load_or_create(meta_path())?;
    let meta_current_folder = meta.root_folders_mut().get_or_create(folder_path)?;

    let app = Box::new(gui::FileManagerApp::new(images, meta, meta_current_folder));
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

fn meta_path() -> PathBuf {
    let dirs = ProjectDirs::from("net", "JP", "JP File Manager");
    if let Some(dirs) = dirs.filter(|_| !cfg!(debug_assertions)) {
        dirs.config_local_dir().to_owned()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(REPOSITORY_PATH)
    }
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

    #[error("problem accessing meta data: {0}")]
    MetaError(#[from] meta::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
