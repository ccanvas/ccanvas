use std::{
    io::Write,
    sync::{
        mpsc::{self, Sender},
        OnceLock,
    },
    thread::{self, JoinHandle},
};

use mio::net::UnixStream;

use crate::Connection;

pub struct MessageThread;

// control code, target, data
static SENDER: OnceLock<Sender<(MessageTarget, Vec<u8>)>> = OnceLock::new();

pub enum MessageTarget {
    One(usize),
    Multiple(Vec<usize>),
    PathStr(String),
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
                    MessageTarget::PathStr(path) => {
                        if let Ok(mut sock) = UnixStream::connect(path) {
                            let _ = sock.write_all(&bytes);
                        }
                    }
                }
            }
        })
    }

    pub fn sender() -> &'static Sender<(MessageTarget, Vec<u8>)> {
        SENDER.get().unwrap()
    }
}
