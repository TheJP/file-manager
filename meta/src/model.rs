use crate::Result;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, Copy, Clone)]
pub struct PersonId(pub usize);

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub pronouns: Option<String>,
    pub comment: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonCollection {
    pub(crate) next_id: PersonId,
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

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Folder {
    pub(crate) path: PathBuf,
    pub(crate) root_folder: RootFolderId,
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) files: HashMap<PathBuf, MetaFile>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaFile {
    pub hash: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub persons: HashSet<PersonId>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, Vec<String>>,
}

impl PersonCollection {
    pub fn entries(&self) -> &HashMap<PersonId, Person> {
        &self.persons
    }

    pub fn add(&mut self, person: Person) -> PersonId {
        let id = self.next_id;
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

    pub fn get_or_create(&mut self, path: impl AsRef<Path>) -> Result<RootFolderId> {
        let path = path.as_ref().canonicalize()?;
        let root_folder = self
            .root_folders
            .iter()
            .find_map(|(&id, root_folder)| path.starts_with(root_folder).then_some(id));

        Ok(root_folder.unwrap_or_else(|| {
            let id = self.next_id;
            self.root_folders.insert(id, path);
            self.next_id.0 += 1;
            id
        }))
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
