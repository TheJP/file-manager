use crate::{Error, Result};

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use crossbeam_channel::{bounded, Receiver};
use egui_extras::RetainedImage;
use image::ImageFormat;
use rayon::{ThreadPool, ThreadPoolBuilder};
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
        values: Default::default(),
        pool: ThreadPoolBuilder::new().build()?,
        processing: Default::default(),
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

pub(crate) struct ImageCache {
    paths: Vec<(usize, PathBuf)>,
    current_image: Option<isize>,
    values: HashMap<usize, RetainedImage>,
    pool: ThreadPool,

    /// Contains connections to all images that are currently being loaded.
    processing: HashMap<usize, Receiver<std::result::Result<RetainedImage, ChannelError>>>,
}

impl ImageCache {
    const PRELOAD_BEFORE: isize = 3;
    const PRELOAD_AFTER: isize = 10;

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
        let key = *key;

        #[allow(clippy::map_entry)]
        if !self.values.contains_key(&key) {
            if !self.processing.contains_key(&key) {
                self.start_loading_image(key, path.clone());
            }
            let image = self.processing[&key]
                .recv()
                .expect("channel empty or disconnected")?;
            self.values.insert(key, image);
            self.processing.remove(&key);
        }

        // Start preloading after retrieving requested image to make loading the requested image a priority.
        self.preload(index);

        Ok(Some(&self.values[&key]))
    }

    fn preload(&mut self, around: isize) {
        for index in (around - Self::PRELOAD_BEFORE)..(around + Self::PRELOAD_AFTER) {
            let Some(index) = index.checked_rem_euclid(self.paths.len() as isize) else {
                continue;
            };
            let (key, path) = &self.paths[index as usize];

            if !self.values.contains_key(key) && !self.processing.contains_key(key) {
                self.start_loading_image(*key, path.clone());
            }
        }
    }

    fn start_loading_image(&mut self, key: usize, path: PathBuf) {
        let (sender, receiver) = bounded(1);
        self.processing.insert(key, receiver);

        self.pool.spawn(move || {
            let image = load_image(path);
            let image = image.map_err(|error| ChannelError(format!("{error}"))); // `std::error:Error` does not implement `Send`.
            sender.send(image).expect("channel disconnected");
        });
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct ChannelError(String);
