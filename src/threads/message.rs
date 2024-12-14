use std::{io::Write, sync::{
    mpsc::{self, Sender},
    OnceLock,
}, thread::{self, JoinHandle}};

use mio::net::UnixStream;

use crate::Connection;

pub struct MessageThread;

// control code, target, data
static SENDER: OnceLock<Sender<(MessageTarget, Vec<u8>)>> = OnceLock::new();

pub enum MessageTarget {
    One(usize),
    Multiple(Vec<usize>)
}

impl MessageThread {
    pub fn spawn() -> JoinHandle<()> {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx).unwrap();

        thread::spawn(|| {
            for (target, bytes) in rx {
                match target {
                    MessageTarget::One(id) => Self::write_stream(Connection::get_mut(&id).unwrap(), &bytes),
                    MessageTarget::Multiple(ids) => ids.iter().for_each(|id| Self::write_stream(Connection::get_mut(id).unwrap(), &bytes)),
                }
            }
        })
    }

    fn write_stream(connection: &mut Connection, bytes: &[u8]) {
        if let Some(stream) = connection.client.as_mut() {
            if stream.write_all(bytes).is_err() {
                connection.client = None;
            }
        }
    }
}
