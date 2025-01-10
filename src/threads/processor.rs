use ccanvas_bindings::{
    packets::{ApprConn, PacketReq, PacketRes, ReqConn},
    rmp_serde,
};
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
                        let deser = rmp_serde::from_slice::<PacketReq>(&data).unwrap();

                        println!("got={deser:?}");

                        match deser {
                            PacketReq::ReqConn(ReqConn { socket, label, parent }) => {
                                if let Err(e) = Connection::create(token.0, &socket, &parent, label) {
                                    if let Some(socket) = socket {
                                        Self::message(
                                            MessageTarget::Path(socket),
                                            &PacketRes::RejConn(e),
                                        )
                                    }
                                } else if socket.is_some() {
                                    Self::message(
                                        MessageTarget::One(token.0),
                                        &PacketRes::ApprConn(ApprConn),
                                    );
                                }
                            }
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

    fn ser(res: &PacketRes) -> Vec<u8> {
        rmp_serde::to_vec_named(res).unwrap()
    }

    fn message(target: MessageTarget, res: &PacketRes) {
        println!("sent={res:?}");
        MessageThread::sender()
            .send((target, Self::ser(res)))
            .unwrap();
    }
}
