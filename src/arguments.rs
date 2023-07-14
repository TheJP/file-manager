use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "JPFileManager", author = "JP")]
#[command(about = "App for organising files")]
#[command(version = None, long_about = None)]
pub(crate) struct Arguments {
    /// Folder containing files that should be organised.
    pub(crate) folder: PathBuf,
}
