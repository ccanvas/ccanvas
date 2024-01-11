use std::collections::{hash_map::Entry, HashMap};

use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::structs::{Discriminator, EventSerde, Response, ResponseContent};

/// pool of key-value pairs for shared or private access
#[derive(Default)]
pub struct Pool {
    map: HashMap<String, PoolItem>,
}

impl Pool {
    /// get a key value, similar to a regular hashap
    pub fn get(&mut self, label: &str) -> Option<Value> {
        self.map.get(label).map(PoolItem::value).map(Value::clone)
    }

    /// set a value, this will call all the watchers
    pub fn set(&mut self, label: &str, value: Value) {
        match self.map.entry(label.to_string()) {
            Entry::Occupied(mut entry) => entry.get_mut().set(label, value),
            Entry::Vacant(entry) => {
                let _ = entry.insert(PoolItem::new(value));
            }
        }
    }

    /// remove a value, this will call all the watchers
    /// return false if no such entry exist
    pub fn remove(&mut self, label: &str, discrim: &Discriminator) -> bool {
        match self.map.remove(label) {
            Some(entry) => entry.remove(label, discrim),
            None => return false,
        }

        true
    }

    /// insert a watch to an entry
    /// return false if no such entry exist
    pub fn watch(
        &mut self,
        label: &str,
        listener: UnboundedSender<Response>,
        discrim: Discriminator,
    ) -> bool {
        match self.map.get_mut(label) {
            Some(entry) => entry.watch(discrim, listener),
            None => return false,
        }

        true
    }

    /// remove watche from entry
    pub fn unwatch(
        &mut self,
        label: &str,
        discrim: &Discriminator,
        self_discrim: Discriminator,
    ) -> bool {
        self.map.get_mut(label).is_some_and(|entry| {
            entry.unwatch(discrim, label.to_string(), self_discrim);
            true
        })
    }
}

/// a single entry in pool
pub struct PoolItem {
    /// the actual value of the entry
    value: Value,
    /// watchers of the entry
    listener: HashMap<Discriminator, UnboundedSender<Response>>,
}

impl PoolItem {
    /// create new self with a value
    pub fn new(value: Value) -> Self {
        Self {
            value,
            listener: HashMap::new(),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    /// add watcher to self
    pub fn watch(&mut self, discrim: Discriminator, sender: UnboundedSender<Response>) {
        self.listener.insert(discrim, sender);
    }

    /// remove watcher from self
    pub fn unwatch(&mut self, discrim: &Discriminator, label: String, self_discrim: Discriminator) {
        if let Some(watcher) = self.listener.remove(discrim) {
            let _ = watcher.send(Response::new(ResponseContent::Event {
                content: EventSerde::ValueRemoved {
                    label,
                    discrim: self_discrim,
                },
            }));
        }
    }

    /// create an entry, override existing value
    /// never fails
    /// call all watchers and remove the dead ones
    pub fn set(&mut self, label: &str, value: Value) {
        self.value = value;

        let mut dead_senders = Vec::new();
        self.listener.iter().for_each(|(discrim, listener)| {
            if listener
                .send(Response::new(ResponseContent::Event {
                    content: EventSerde::ValueUpdated {
                        label: label.to_string(),
                        new: self.value.clone(),
                        discrim: discrim.clone(),
                    },
                }))
                .is_err()
            {
                dead_senders.push(discrim.clone())
            }
        });

        dead_senders.iter().for_each(|key| {
            let _ = self.listener.remove(key);
        })
    }

    /// remove an entry
    pub fn remove(&self, label: &str, self_discrim: &Discriminator) {
        self.listener.values().for_each(|listener| {
            let _ = listener.send(Response::new(ResponseContent::Event {
                content: EventSerde::ValueRemoved {
                    label: label.to_string(),
                    discrim: self_discrim.clone(),
                },
            }));
        })
    }
}
