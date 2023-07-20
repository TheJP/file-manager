use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use model::{Folder, MetaFile, PersonCollection, RootFolderCollection, RootFolderId};
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

    /// Cache of folder data. Not all folder data is loaded when setting up the repository.
    folders: HashMap<PathBuf, Folder>,
}

// TODO: Add write-lock file to data_path.
// TODO: Add short checksum to folder meta data and data_path
//       to check if the used ids actually reference our data_path or an other data_path.
//       Prevents user error from causing too many inconsistencies.
impl Repository {
    const PERSONS_FILENAME: &str = "persons.json";
    const ROOT_FOLDERS_FILENAME: &str = "root_folders.json";
    const FOLDER_FILENAME: &str = ".jpfolder.json";

    pub fn load_or_create(data_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_path)?;

        let persons = read_or_create(&data_path, Self::PERSONS_FILENAME)?;
        let root_folders = read_or_create(&data_path, Self::ROOT_FOLDERS_FILENAME)?;

        Ok(Self {
            data_path,
            persons,
            root_folders,
            folders: HashMap::with_capacity(0),
        })
    }

    pub fn save(&self) -> Result<()> {
        self.save_persons()?;
        self.save_root_folders()?;
        self.save_file_data()?;
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

    pub fn save_file_data(&self) -> Result<()> {
        for folder in self.folders.values() {
            write(&folder.path, Self::FOLDER_FILENAME, folder)?;
        }
        Ok(())
    }

    pub fn load_or_create_file(
        &mut self,
        root_folder_id: &RootFolderId,
        path: impl AsRef<Path>,
    ) -> Result<&mut MetaFile> {
        let root_folder = self
            .root_folders
            .root_folder(root_folder_id)
            .ok_or(Error::InvalidRootFolder)?;
        let absolute_path = path.as_ref().canonicalize()?;
        if !absolute_path.starts_with(root_folder) {
            return Err(Error::FileNotInRootFolder);
        }

        let folder_path = absolute_path.parent().ok_or(Error::InvalidRootFolder)?;
        let folder = Self::load_or_create_folder(&mut self.folders, folder_path, root_folder_id)?;

        let file_name = absolute_path
            .file_name()
            .ok_or(Error::InvalidFilePath)?
            .to_os_string();
        Ok(folder
            .files
            .entry(file_name.into())
            .or_insert_with(|| MetaFile {
                hash: None,
                persons: HashSet::with_capacity(0),
                tags: HashMap::with_capacity(0),
            }))
    }

    fn load_or_create_folder<'a>(
        folder_cache: &'a mut HashMap<PathBuf, Folder>,
        path: impl AsRef<Path>,
        root_folder_id: &RootFolderId,
    ) -> Result<&'a mut Folder> {
        let path = path.as_ref();
        let owned_path = path.to_path_buf();

        if folder_cache.contains_key(&owned_path) {
            return Ok(folder_cache.get_mut(&owned_path).unwrap());
        }

        let meta_path = path.join(Self::FOLDER_FILENAME);
        if meta_path.try_exists()? {
            let meta_file = File::open(meta_path)?;
            let meta_reader = BufReader::new(meta_file);
            let folder = serde_json::from_reader(meta_reader)?;
            folder_cache.insert(owned_path.clone(), folder);
        } else {
            folder_cache.insert(
                owned_path.clone(),
                Folder {
                    path: owned_path.clone(),
                    root_folder: *root_folder_id,
                    files: HashMap::new(),
                },
            );
        }

        Ok(folder_cache.get_mut(&owned_path).unwrap())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("json parsing error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("given root folder does not exist")]
    InvalidRootFolder,

    #[error("given file is not inside provided root folder")]
    FileNotInRootFolder,

    #[error("given path does not contain a file name")]
    InvalidFilePath,
}
pub type Result<T> = std::result::Result<T, Error>;
