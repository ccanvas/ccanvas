use std::collections::HashMap;

use crate::structs::{Subscription, Discriminator};

pub struct Suppressors(HashMap<Subscription, Vec<SuppressItem>>);

impl Suppressors {
    // pub fn insert(subscription: Subscription, item: SuppressItem) -> bool {
    // }
}

pub struct SuppressItem {
    source: Discriminator,
    priority: u32
}

impl SuppressItem {
    pub fn new(source: Discriminator, priority: u32) -> Self {
        Self {}
    }
}
