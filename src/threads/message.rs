use std::{
    sync::{
        mpsc::{self, Sender},
        OnceLock,
    },
    thread::{self, JoinHandle},
};

use crate::Connection;

pub struct MessageThread;

// control code, target, data
static SENDER: OnceLock<Sender<(MessageTarget, Vec<u8>)>> = OnceLock::new();

pub enum MessageTarget {
    One(usize),
    Multiple(Vec<usize>),
}

impl MessageThread {
    pub fn spawn() -> JoinHandle<()> {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx).unwrap();

        thread::spawn(|| {
            for (target, bytes) in rx {
                match target {
                    MessageTarget::One(id) => Connection::get_mut(&id).unwrap().write(&bytes),
                    MessageTarget::Multiple(ids) => ids
                        .iter()
                        .for_each(|id| Connection::get_mut(id).unwrap().write(&bytes)),
                }
            }
        })
    }
}
