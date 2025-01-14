use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use crate::{MessageTarget, MessageThread};

// editable by PROCESSOR
// vec stores the same content as hashset just less searchable
#[allow(clippy::type_complexity)]
static mut SUBSCRIPTIONS: OnceLock<HashMap<Vec<u8>, (HashSet<usize>, Vec<usize>)>> =
    OnceLock::new();
static mut USER_SUBSCRIPTIONS: OnceLock<HashMap<usize, HashSet<Vec<u8>>>> = OnceLock::new();

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
        if let Some((_, sub)) = unsafe { SUBSCRIPTIONS.get_mut() }.unwrap().remove(channel) {
            let user_sub = unsafe { USER_SUBSCRIPTIONS.get_mut() }.unwrap();
            sub.iter().for_each(|id| {
                let user = user_sub.get_mut(id).unwrap();
                user.remove(channel);
                if user.is_empty() {
                    user_sub.remove(id);
                }
            });

            // TODO broadcast destroy channel
            true
        } else {
            false
        }
    }

    fn notify(channel: &[u8], bytes: Vec<u8>) -> bool {
        if let Some((_, targets)) = Self::get(channel) {
            MessageThread::sender()
                .send((MessageTarget::Multiple(targets.clone()), bytes))
                .unwrap();
            true
        } else {
            false
        }
    }

    fn subscribe(channel: &[u8], id: usize) -> bool {
        if let Some((set, vec)) = unsafe { SUBSCRIPTIONS.get_mut() }.unwrap().get_mut(channel) {
            let user_sub = unsafe { USER_SUBSCRIPTIONS.get_mut() }.unwrap();
            if let Some(user) = user_sub.get_mut(&id) {
                if user.contains(channel) {
                    return true;
                }
                user.insert(channel.to_vec());
            } else {
                user_sub.insert(id, HashSet::from([channel.to_vec()]));
            }

            set.insert(id);
            vec.push(id);
            true
        } else {
            false
        }
    }

    fn subscribe_force(channel: &[u8], id: usize) {
        let sub = unsafe { SUBSCRIPTIONS.get_mut() }.unwrap();
        if !sub.contains_key(channel) {
            sub.insert(channel.to_vec(), (HashSet::new(), Vec::new()));
        }

        Self::subscribe(channel, id);
    }

    fn unsubscribe(channel: &[u8], id: usize) -> bool {
        if let Some((set, vec)) = unsafe { SUBSCRIPTIONS.get_mut() }.unwrap().get_mut(channel) {
            if !set.remove(&id) {
                return false;
            }

            let user_sub = unsafe { USER_SUBSCRIPTIONS.get_mut() }.unwrap();
            let user = user_sub.get_mut(&id).unwrap();
            user.remove(channel);

            if channel.is_empty() {
                user_sub.remove(&id);
            }

            vec.swap_remove(vec.iter().position(|x| x == &id).unwrap());
            true
        } else {
            false
        }
    }
}
