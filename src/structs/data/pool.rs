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
        self.map
            .get(label)
            .map(|item| item.value.as_ref().unwrap().clone())
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
    pub fn remove(&mut self, label: &str, discrim: &Discriminator) {
        if let Some(entry) = self.map.get_mut(label) {
            entry.remove(label, discrim);
            if entry.is_empty() {
                self.map.remove(label);
            }
        }
    }

    /// insert a watch to an entry
    /// return false if no such entry exist
    pub fn watch(
        &mut self,
        label: String,
        listener: UnboundedSender<Response>,
        discrim: Discriminator,
    ) {
        self.map.entry(label).or_default().watch(discrim, listener);
    }

    /// remove watche from entry
    pub fn unwatch(
        &mut self,
        label: &str,
        discrim: &Discriminator,
        self_discrim: Discriminator,
    ) -> bool {
        match self.map.get_mut(label) {
            Some(entry) => {
                entry.unwatch(discrim, label.to_string(), self_discrim);

                if entry.is_empty() {
                    self.map.remove(label);
                }
                true
            }
            None => false,
        }
    }
}

#[derive(Default)]
/// a single entry in pool
pub struct PoolItem {
    /// the actual value of the entry
    value: Option<Value>,
    /// watchers of the entry
    listener: HashMap<Discriminator, UnboundedSender<Response>>,
}

impl PoolItem {
    /// create new self with a value
    pub fn new(value: Value) -> Self {
        Self {
            value: Some(value),
            listener: HashMap::new(),
        }
    }

    pub fn value(&self) -> &Option<Value> {
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
        self.value = Some(value.clone());

        let mut dead_senders = Vec::new();
        self.listener.iter().for_each(|(discrim, listener)| {
            if listener
                .send(Response::new(ResponseContent::Event {
                    content: EventSerde::ValueUpdated {
                        label: label.to_string(),
                        new: value.clone(),
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
    pub fn remove(&mut self, label: &str, self_discrim: &Discriminator) {
        self.value = None;

        self.listener.values().for_each(|listener| {
            let _ = listener.send(Response::new(ResponseContent::Event {
                content: EventSerde::ValueRemoved {
                    label: label.to_string(),
                    discrim: self_discrim.clone(),
                },
            }));
        })
    }

    /// returns true if there is absolutely nothing to keep in this entry
    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.listener.is_empty()
    }
}
