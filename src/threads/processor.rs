use std::sync::{mpsc::{self, Sender}, OnceLock};

#[derive(Debug)]
pub enum ProcessorEvent {
    Packet {
        source: usize,
        data: Vec<u8>
    }
}

static SENDER: OnceLock<Sender<ProcessorEvent>> = OnceLock::new();

pub struct ProcessorThread;

impl ProcessorThread {
    pub fn spawn() {
        let (tx, rx) = mpsc::channel();
        SENDER.set(tx);

        for event in rx.recv() {
            dbg!(event);
        }
    }

    pub fn sender() -> Sender<ProcessorEvent> {
        SENDER.get().unwrap().clone()
    }
}
