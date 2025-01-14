use std::{
    cmp,
    collections::HashMap,
    io::{self, BorrowedBuf, Read},
    sync::{mpsc::Sender, OnceLock},
    thread::{self, JoinHandle},
};

use mio::{
    event::Event,
    net::{UnixListener, UnixStream},
    Events, Interest, Poll, Registry, Token,
};

use crate::{threads::processor::ProcessorThread, Instance};

use super::ProcessorEvent;

pub struct ConnectionThread;

static REGISTRY: OnceLock<&Registry> = OnceLock::new();

impl ConnectionThread {
    pub fn spawn() -> JoinHandle<()> {
        let mut listener = UnixListener::bind(Instance::sock_create()).unwrap();
        static mut POLL: OnceLock<Poll> = OnceLock::new();
        unsafe { POLL.set(Poll::new().unwrap()) }.unwrap();
        REGISTRY
            .set(unsafe { POLL.get() }.unwrap().registry())
            .unwrap();

        REGISTRY
            .get()
            .unwrap()
            .register(&mut listener, Token(0), Interest::READABLE)
            .unwrap();

        thread::spawn(move || {
            let mut connections: HashMap<Token, UnixStream> = HashMap::new();
            let processor = ProcessorThread::sender();

            let mut connection_token = Token(1);
            let poll = unsafe { POLL.get_mut() }.unwrap();
            let mut events = Events::with_capacity(1024);

            loop {
                let res = poll.poll(&mut events, None);
                if let Err(err) = &res {
                    if err.kind() == io::ErrorKind::Interrupted {
                        continue;
                    } else {
                        res.unwrap();
                    }
                }

                for event in events.iter() {
                    match event.token() {
                        Token(0) => loop {
                            let (mut connection, _) = match listener.accept() {
                                Ok(res) => res,
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    break;
                                }
                                e => {
                                    e.unwrap();
                                    unreachable!()
                                }
                            };

                            REGISTRY
                                .get()
                                .unwrap()
                                .register(&mut connection, connection_token, Interest::READABLE)
                                .unwrap();
                            connections.insert(connection_token, connection);

                            connection_token.0 += 1;
                        },
                        token => {
                            if let Some(connection) = connections.get_mut(&token) {
                                let done =
                                    handle_event(connection, event, &token, &processor).unwrap();
                                if done {
                                    REGISTRY.get().unwrap().deregister(connection).unwrap();
                                    connections.remove(&token);
                                    processor
                                        .send(ProcessorEvent::Disconnect { token: token.0 })
                                        .unwrap();
                                }
                            } else {
                                continue;
                            };
                        }
                    }
                }
            }
        })
    }

    pub fn registry() -> &'static Registry {
        REGISTRY.get().unwrap()
    }

    pub fn add_server(listener: &mut UnixListener, id: usize) {
        ConnectionThread::registry()
            .register(listener, Token(id), Interest::READABLE)
            .unwrap();
    }
}

fn handle_event(
    connection: &mut UnixStream,
    event: &Event,
    token: &Token,
    sender: &Sender<ProcessorEvent>,
) -> io::Result<bool> {
    if event.is_readable() {
        let mut recieved_data = Vec::new();

        let _ = default_read_to_end(connection, &mut recieved_data, None);

        if !recieved_data.is_empty() {
            sender
                .send(ProcessorEvent::Packet {
                    data: recieved_data,
                    token: *token,
                })
                .unwrap();
            return Ok(false);
        }
        return Ok(true);
    }

    Ok(false)
}

// reimplementation of `read_to_end` that returns the read content
// when WouldBlock is encountered
fn default_read_to_end<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut Vec<u8>,
    size_hint: Option<usize>,
) -> io::Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();
    // Optionally limit the maximum bytes read on each iteration.
    // This adds an arbitrary fiddle factor to allow for more data than we expect.
    let max_read_size = size_hint.and_then(|s| s.checked_add(1024)?.checked_next_power_of_two());

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32); // buf is full, need more space
        }

        let mut spare = buf.spare_capacity_mut();
        if let Some(size) = max_read_size {
            let len = cmp::min(spare.len(), size);
            spare = &mut spare[..len]
        }
        let mut read_buf: BorrowedBuf<'_> = spare.into();

        // SAFETY: These bytes were initialized but not filled in the previous loop
        unsafe {
            read_buf.set_init(initialized);
        }

        let mut cursor = read_buf.unfilled();
        match r.read_buf(cursor.reborrow()) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }

        if cursor.written() == 0 {
            return Ok(buf.len() - start_len);
        }

        // store how much was initialized but not filled
        initialized = cursor.init_ref().len();

        // SAFETY: BorrowedBuf's invariants mean this much memory is initialized.
        unsafe {
            let new_len = read_buf.filled().len() + buf.len();
            buf.set_len(new_len);
        }

        if buf.len() == buf.capacity() && buf.capacity() == start_cap {
            // The buffer might be an exact fit. Let's read into a probe buffer
            // and see if it returns `Ok(0)`. If so, we've avoided an
            // unnecessary doubling of the capacity. But if not, append the
            // probe buffer to the primary buffer and let its capacity grow.
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(buf.len() - start_len),
                    Ok(n) => {
                        buf.extend_from_slice(&probe[..n]);
                        break;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
