use std::{
    sync::OnceLock,
    thread::{self, JoinHandle},
};

use mio::{Events, Poll, Registry};

pub struct ConnectionThread;

static REGISTRY: OnceLock<&Registry> = OnceLock::new();

impl ConnectionThread {
    pub fn spawn() -> JoinHandle<()> {
        static mut POLL: OnceLock<Poll> = OnceLock::new();
        unsafe { POLL.set(Poll::new().unwrap()) }.unwrap();
        REGISTRY
            .set(unsafe { POLL.get() }.unwrap().registry())
            .unwrap();

        thread::spawn(|| {
            let poll = unsafe { POLL.get_mut() }.unwrap();
            let mut events = Events::with_capacity(1024);

            loop {
                poll.poll(&mut events, None).unwrap();

                for event in &events {
                    // send to processor
                }
            }
        })
    }

    pub fn registry() -> &'static Registry {
        REGISTRY.get().unwrap()
    }
}
