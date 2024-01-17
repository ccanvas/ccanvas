use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

static DISCRIM: OnceCell<Mutex<u32>> = OnceCell::const_new_with(Mutex::new(0));
/// get a unique discriminator chunk
pub fn discrim() -> u32 {
    let mut discrim = DISCRIM.get().unwrap().lock().unwrap();
    *discrim += 1;
    *discrim
}

/// a unique path id for every component
#[derive(Default, PartialEq, Eq, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Discriminator(pub Vec<u32>);

impl Discriminator {
    /// create new child component
    pub fn new_child(&self) -> Self {
        let mut new_discrim = self.0.to_vec();
        new_discrim.push(discrim());
        Self(new_discrim)
    }

    /// returns internal vec
    pub fn as_vec(&self) -> &Vec<u32> {
        &self.0
    }

    /// check if one component is a child of another
    pub fn is_parent_of(&self, other: &Self) -> bool {
        other.0.starts_with(&self.0) && self.0.len() < other.0.len()
    }

    /// truncate path length
    pub fn truncate(mut self, len: usize) -> Self {
        self.0.truncate(len);
        self
    }

    /// return immediate chaild to pass to
    pub fn immediate_child(&self, child: Self) -> Option<Self> {
        self.is_parent_of(&child)
            .then(|| child.truncate(self.0.len() + 1))
    }

    /// returns the immediate parent
    /// None if component is top level [1]
    pub fn immediate_parent(self) -> Option<Self> {
        (!self.0.is_empty()).then(|| {
            let len = self.0.len();
            self.truncate(len - 1)
        })
    }

    /// returns discriminator of master space
    pub fn master() -> Self {
        Self(vec![1])
    }

    /// check if discriminator is empty (would be invalid)
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// check if self starts with other
    pub fn starts_with(&self, other: &Self) -> bool {
        self.0.starts_with(&other.0)
    }
}
