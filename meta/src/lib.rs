use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use model::{PersonCollection, RootFolderCollection};
use serde::Serialize;

pub mod model;

fn read_or_create<T>(path: &Path, file_name: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    let path = path.join(file_name);
    if !path.try_exists()? {
        Ok(Default::default())
    } else {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

fn write<T>(path: &Path, file_name: impl AsRef<Path>, value: &T) -> Result<()>
where
    T: ?Sized + Serialize,
{
    let path = path.join(file_name);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.flush()?)
}

pub struct Repository {
    data_path: PathBuf,
    persons: PersonCollection,
    root_folders: RootFolderCollection,
}

impl Repository {
    const PERSONS_FILENAME: &str = "persons.json";
    const ROOT_FOLDERS_FILENAME: &str = "root_folders.json";

    pub fn load_or_create(data_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_path)?;

        let persons = read_or_create(&data_path, Self::PERSONS_FILENAME)?;
        let root_folders = read_or_create(&data_path, Self::ROOT_FOLDERS_FILENAME)?;

        Ok(Self {
            data_path,
            persons,
            root_folders,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.save_persons()?;
        self.save_root_folders()?;
        Ok(())
    }

    pub fn persons(&self) -> &PersonCollection {
        &self.persons
    }

    pub fn persons_mut(&mut self) -> &mut PersonCollection {
        &mut self.persons
    }

    pub fn save_persons(&self) -> Result<()> {
        write(&self.data_path, Self::PERSONS_FILENAME, &self.persons)
    }

    pub fn root_folders(&self) -> &RootFolderCollection {
        &self.root_folders
    }

    pub fn root_folders_mut(&mut self) -> &mut RootFolderCollection {
        &mut self.root_folders
    }

    pub fn save_root_folders(&self) -> Result<()> {
        write(
            &self.data_path,
            Self::ROOT_FOLDERS_FILENAME,
            &self.root_folders,
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("json parsing error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
