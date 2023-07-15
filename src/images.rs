use crate::{Error, Result};

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use egui_extras::RetainedImage;
use image::ImageFormat;
use walkdir::WalkDir;

pub(crate) fn find(folder_path: impl AsRef<Path>) -> Result<ImageCache> {
    let mut paths = Vec::new();
    let mut index = 0;

    for file in WalkDir::new(folder_path).sort_by_file_name() {
        let file = file?;
        let Ok(format) = ImageFormat::from_path(file.path()) else { continue; };
        if format.can_read() {
            paths.push((index, file.into_path()));
            index += 1;
        }
    }

    Ok(ImageCache {
        paths,
        current_image: (index > 0).then_some(0),
        ..Default::default()
    })
}

fn load_image(path: impl AsRef<Path>) -> Result<RetainedImage> {
    use std::io::prelude::Read;
    let mut bytes = Vec::new();
    File::open(&path)?.read_to_end(&mut bytes)?;

    let debug_name = path
        .as_ref()
        .file_name()
        .map_or("[Image]".into(), |name| name.to_string_lossy());

    RetainedImage::from_image_bytes(debug_name, &bytes).map_err(Error::DisplayImage)
}

#[derive(Default)]
pub(crate) struct ImageCache {
    paths: Vec<(usize, PathBuf)>,
    current_image: Option<isize>,
    values: HashMap<usize, RetainedImage>,
}

impl ImageCache {
    pub(crate) fn forward(&mut self) {
        self.move_by(1);
    }

    pub(crate) fn back(&mut self) {
        self.move_by(-1);
    }

    pub(crate) fn move_by(&mut self, step: isize) {
        self.current_image = self
            .current_image
            .and_then(|index| (index + step).checked_rem_euclid(self.paths.len() as isize));
    }

    pub(crate) fn current_image(&mut self) -> Result<Option<&RetainedImage>> {
        let Some(index) = self.current_image else {
            return Ok(None);
        };
        let (key, path) = &self.paths[index as usize];

        if !self.values.contains_key(key) {
            let image = load_image(path)?;
            self.values.insert(*key, image);
        }

        Ok(Some(&self.values[key]))
    }
}
