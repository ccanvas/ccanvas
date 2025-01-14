use ccanvas_bindings::packets::{connection, Packet};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        OnceLock,
    },
    thread::{self, JoinHandle},
};

use mio::Token;

use crate::{Connection, MessageTarget, MessageThread};

#[derive(Debug)]
pub enum ProcessorEvent {
    Packet { token: Token, data: Vec<u8> },
    Disconnect { token: usize },
}

static SENDER: OnceLock<Sender<ProcessorEvent>> = OnceLock::new();

pub struct ProcessorThread;

impl ProcessorThread {
    pub fn start(rx: Receiver<ProcessorEvent>) {
        while let Ok(event) = rx.recv() {
            match event {
                ProcessorEvent::Packet { token, data } => {
                    let deser = match Packet::from_bytes(&data) {
                        Some(deser) => deser,
                        None => continue,
                    };

                    match deser {
                        Packet::Connection(connection::Group::ReqConn {
                            socket: Some((path, echo)),
                            label,
                        }) => {
                            if Connection::create(token.0, Some(&path), label) {
                                Self::message(
                                    MessageTarget::One(token.0),
                                    Packet::Connection(connection::Group::ApprConn { echo }),
                                );
                            } else {
                                Self::message(
                                    MessageTarget::PathStr(path),
                                    Packet::Connection(connection::Group::RejConn { echo }),
                                )
                            }
                        }
                        Packet::Connection(connection::Group::ReqConn {
                            socket: None,
                            label,
                        }) => {
                            Connection::create(token.0, None, label);
                        }
                        Packet::Connection(connection::Group::Terminate) => {
                            return;
                        }
                        _ => {}
                    }
                }
                ProcessorEvent::Disconnect { token } => {
                    Connection::remove_id(token);
                }
            }
        }
    }

    pub fn init() -> Receiver<ProcessorEvent> {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx).unwrap();
        rx
    }

    pub fn sender() -> Sender<ProcessorEvent> {
        SENDER.get().unwrap().clone()
    }

    fn message(target: MessageTarget, res: Packet) {
        MessageThread::sender()
            .send((target, res.to_bytes()))
            .unwrap();
    }
}
