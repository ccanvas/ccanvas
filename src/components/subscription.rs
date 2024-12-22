use std::{collections::{HashMap, HashSet}, sync::OnceLock};

use crate::{MessageTarget, MessageThread};

// editable by PROCESSOR
// vec stores the same content as hashset just less searchable
static mut SUBSCRIPTIONS: OnceLock<HashMap<Vec<u8>, (HashSet<usize>, Vec<usize>)>> = OnceLock::new();

pub struct Subscription;

impl Subscription {
    fn get(channel: &[u8]) -> Option<&(HashSet<usize>, Vec<usize>)> {
        unsafe { SUBSCRIPTIONS.get() }.unwrap().get(channel)
    }

    fn create(channel: &[u8]) -> bool {
        let map = unsafe { SUBSCRIPTIONS.get_mut() }.unwrap();

        if map.contains_key(channel) {
            false
        } else {
            map.insert(channel.to_vec(), (HashSet::new(), Vec::new()));
            // TODO broadcast create channel
            true
        }
    }

    fn destroy(channel: &[u8]) -> bool {
        unsafe { SUBSCRIPTIONS.get_mut() }.unwrap().remove(channel).is_some()
        // TODO broadcast destroy channel
    }

    fn notify(channel: &[u8], bytes: Vec<u8>) -> bool {
        if let Some((_, targets)) = Self::get(channel) {
            MessageThread::sender().send((MessageTarget::Multiple(targets.clone()), bytes)).unwrap();
            true
        } else {
            false
        }
    }
}
