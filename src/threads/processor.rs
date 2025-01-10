use std::{
    sync::{
        mpsc::{self, Sender},
        OnceLock,
    },
    thread::{self, JoinHandle},
};

use mio::Token;

#[derive(Debug)]
pub enum ProcessorEvent {
    Packet {
        token: Token,
        data: Vec<u8>
    },
}

static SENDER: OnceLock<Sender<ProcessorEvent>> = OnceLock::new();

pub struct ProcessorThread;

impl ProcessorThread {
    pub fn spawn() -> JoinHandle<()> {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx).unwrap();

        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                dbg!(event);
            }

            panic!("recv shutdown")
        })
    }

    pub fn sender() -> Sender<ProcessorEvent> {
        SENDER.get().unwrap().clone()
    }
}
