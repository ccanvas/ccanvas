use std::{collections::HashMap, sync::Mutex};

use tokio::sync::OnceCell;

use crate::structs::Subscription;

#[derive(Default)]
pub struct Suppressors {
    map: HashMap<Subscription, Vec<SuppressItem>>,
    state_id: u32,
}

impl Suppressors {
    pub fn insert(&mut self, subscription: Subscription, priority: u32) -> u32 {
        self.state_id += 1;
        let channel = self.map.entry(subscription).or_default();
        let item = SuppressItem::new(priority);
        let id = item.id;

        for (i, suppressor) in channel.iter().enumerate() {
            if suppressor.priority < item.priority {
                channel.insert(i, item);
                return id;
            }
        }

        channel.push(item);
        id
    }

    pub fn remove(&mut self, subscription: Subscription, id: u32) -> bool {
        self.state_id += 1;
        if let Some(channel) = self.map.get_mut(&subscription) {
            if let Some(index) = channel.iter().position(|item| item.id == id) {
                channel.remove(index);
                if channel.is_empty() {
                    self.map.remove(&subscription);
                }

                return true;
            }
        }

        false
    }

    pub fn suppress_level(&self, channels: &[Subscription]) -> Option<u32> {
        channels
            .iter()
            .filter_map(|channel| {
                self.map
                    .get(channel)
                    .map(|item| item.first().unwrap().priority)
            })
            .max()
    }

    pub fn state_id(&self) -> u32 {
        self.state_id
    }
}

pub struct SuppressItem {
    pub priority: u32,
    pub id: u32,
}

static SUPPRESSOR_ID: OnceCell<Mutex<u32>> = OnceCell::const_new_with(Mutex::new(0));

fn gen_id() -> u32 {
    let mut id = SUPPRESSOR_ID.get().unwrap().lock().unwrap();
    *id += 1;
    *id
}

impl SuppressItem {
    pub fn new(priority: u32) -> Self {
        Self {
            priority,
            id: gen_id(),
        }
    }
}
