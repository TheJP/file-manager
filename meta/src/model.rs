use crate::Result;

use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, Copy, Clone)]
pub struct PersonId(usize);

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub(crate) id: PersonId,
    pub name: String,
    pub pronouns: Option<String>,
    pub comment: Option<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, Vec<String>>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonCollection {
    pub(crate) next_id: PersonId,
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) persons: HashMap<PersonId, Person>,

    /// This contains tags that have been previously used for a [Person].
    ///
    /// If a tag is in this collection it does not necessarily have to
    /// currently be set as a tag on any [Person].
    pub used_tags: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, Copy, Clone)]
pub struct RootFolderId(usize);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RootFolderCollection {
    pub(crate) next_id: RootFolderId,
    pub(crate) root_folders: HashMap<RootFolderId, PathBuf>,

    pub file_tags: HashSet<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Folder {
    pub(crate) path: PathBuf,
    pub(crate) root_folder: RootFolderId,
    pub(crate) files: HashMap<OsString, MetaFile>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaFile {
    pub hash: Option<String>,

    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub persons: HashSet<usize>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, Vec<String>>,
}

impl PersonCollection {
    pub fn entries(&self) -> &HashMap<PersonId, Person> {
        &self.persons
    }

    pub fn add(&mut self, mut person: Person) -> PersonId {
        let id = self.next_id;
        person.id = id;
        self.persons.insert(id, person);
        self.next_id.0 += 1;
        id
    }

    pub fn remove(&mut self, id: &PersonId) -> Option<Person> {
        self.persons.remove(id)
    }

    pub fn person(&self, id: &PersonId) -> Option<&Person> {
        self.persons.get(id)
    }

    pub fn person_mut(&mut self, id: &mut PersonId) -> Option<&mut Person> {
        self.persons.get_mut(id)
    }
}

impl Person {
    pub fn new(name: String, pronouns: Option<String>) -> Self {
        Self {
            id: PersonId(usize::MAX),
            name,
            pronouns,
            comment: None,
            tags: HashMap::with_capacity(0),
        }
    }
}

impl RootFolderCollection {
    pub fn entries(&self) -> &HashMap<RootFolderId, PathBuf> {
        &self.root_folders
    }

    pub fn add(&mut self, root_folder: PathBuf) -> Result<RootFolderId> {
        let root_folder = root_folder.canonicalize()?;
        let id = self.next_id;
        self.root_folders.insert(id, root_folder);
        self.next_id.0 += 1;
        Ok(id)
    }

    pub fn remove(&mut self, id: &RootFolderId) -> Option<PathBuf> {
        self.root_folders.remove(id)
    }

    pub fn root_folder(&self, id: &RootFolderId) -> Option<&PathBuf> {
        self.root_folders.get(id)
    }

    pub fn root_folder_mut(&mut self, id: &RootFolderId) -> Option<&mut PathBuf> {
        self.root_folders.get_mut(id)
    }
}
