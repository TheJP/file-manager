use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use model::{Person, PersonCollection};
use serde::Serialize;

pub mod model;

fn read<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

fn write<T>(path: impl AsRef<Path>, value: &T) -> Result<()>
where
    T: ?Sized + Serialize,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.flush()?)
}

pub struct Repository {
    data_path: PathBuf,
    persons: PersonCollection,
}

impl Repository {
    const PERSONS_FILENAME: &str = "persons.json";

    pub fn create_or_load(data_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_path)?;

        let persons_path = data_path.join(Self::PERSONS_FILENAME);
        let persons = read(persons_path)?;

        Ok(Self { data_path, persons })
    }

    pub fn persons(&self) -> &HashMap<usize, Person> {
        &self.persons.persons
    }

    pub fn save_persons(&self) -> Result<()> {
        let persons_path = self.data_path.join(Self::PERSONS_FILENAME);
        write(persons_path, &self.persons)
    }

    pub fn add_person(&mut self, person: Person) -> usize {
        let id = self.persons.next_id;
        self.persons.persons.insert(id, person);
        self.persons.next_id += 1;
        id
    }

    pub fn person(&self, id: &usize) -> Option<&Person> {
        self.persons.persons.get(id)
    }

    pub fn person_mut(&mut self, id: &mut usize) -> Option<&mut Person> {
        self.persons.persons.get_mut(id)
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
