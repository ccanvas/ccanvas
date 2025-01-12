use ccanvas_bindings::packets::{connection, Packet};
use std::{
    sync::{
        mpsc::{self, Sender},
        OnceLock,
    },
    thread::{self, JoinHandle},
};

use mio::Token;

use crate::{Connection, MessageTarget, MessageThread};

#[derive(Debug)]
pub enum ProcessorEvent {
    Packet { token: Token, data: Vec<u8> },
}

static SENDER: OnceLock<Sender<ProcessorEvent>> = OnceLock::new();

pub struct ProcessorThread;

impl ProcessorThread {
    pub fn spawn() -> JoinHandle<()> {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx).unwrap();

        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    ProcessorEvent::Packet { token, data } => {
                        let deser = match Packet::from_bytes(&data) {
                            Some(deser) => deser,
                            None => continue,
                        };

                        println!("got={deser:?}");

                        match deser {
                            Packet::Connection(connection::Group::ReqConn { socket, label }) => {
                                if Connection::create(token.0, &socket, label) {
                                    if socket.is_some() {
                                        Self::message(
                                            MessageTarget::One(token.0),
                                            Packet::Connection(connection::Group::ApprConn),
                                        );
                                    }
                                } else if let Some(socket) = socket {
                                    Self::message(
                                        MessageTarget::PathStr(socket),
                                        Packet::Connection(connection::Group::RejConn),
                                    )
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            panic!("recv shutdown")
        })
    }

    pub fn sender() -> Sender<ProcessorEvent> {
        SENDER.get().unwrap().clone()
    }

    fn message(target: MessageTarget, res: Packet) {
        println!("sent={res:?}");
        MessageThread::sender()
            .send((target, res.to_bytes()))
            .unwrap();
    }
}
