use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) pronouns: Option<String>,
    pub(crate) comment: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonCollection {
    pub(crate) next_id: usize,
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) persons: HashMap<usize, Person>,

    /// This contains tags that have been previously used for a [Person].
    ///
    /// If a tag is in this collection it does not necessarily have to
    /// currently be set as a tag on any [Person].
    pub(crate) used_tags: HashSet<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RootFolderCollection {
    pub(crate) next_id: usize,
    pub(crate) root_folders: HashMap<usize, PathBuf>,
}

impl PersonCollection {
    pub fn entries(&self) -> &HashMap<usize, Person> {
        &self.persons
    }

    pub fn add(&mut self, mut person: Person) -> usize {
        let id = self.next_id;
        person.id = id;
        self.persons.insert(id, person);
        self.next_id += 1;
        id
    }

    pub fn remove(&mut self, id: &usize) -> Option<Person> {
        self.persons.remove(id)
    }

    pub fn person(&self, id: &usize) -> Option<&Person> {
        self.persons.get(id)
    }

    pub fn person_mut(&mut self, id: &mut usize) -> Option<&mut Person> {
        self.persons.get_mut(id)
    }
}

impl RootFolderCollection {
    pub fn entries(&self) -> &HashMap<usize, PathBuf> {
        &self.root_folders
    }

    pub fn add(&mut self, root_folder: PathBuf) -> usize {
        let id = self.next_id;
        self.root_folders.insert(id, root_folder);
        self.next_id += 1;
        id
    }

    pub fn remove(&mut self, id: &usize) -> Option<PathBuf> {
        self.root_folders.remove(id)
    }

    pub fn root_folder(&self, id: &usize) -> Option<&PathBuf> {
        self.root_folders.get(id)
    }

    pub fn root_folder_mut(&mut self, id: &usize) -> Option<&mut PathBuf> {
        self.root_folders.get_mut(id)
    }
}
