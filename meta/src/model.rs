use std::collections::{HashMap, HashSet};

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
#[derive(Debug, Serialize, Deserialize)]
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
