use std::{
    cmp,
    collections::{HashMap, HashSet},
    io::{self, BorrowedBuf, Read},
    sync::{atomic::AtomicUsize, mpsc::Sender, OnceLock},
    thread::{self, JoinHandle},
    usize,
};

use mio::{
    event::Event,
    net::{UnixListener, UnixStream},
    Events, Interest, Poll, Registry, Token,
};

use crate::{threads::processor::ProcessorThread, Connection};

use super::ProcessorEvent;

pub struct ConnectionThread;

static REGISTRY: OnceLock<&Registry> = OnceLock::new();
static mut CONNECTIONS: OnceLock<HashMap<usize, (usize, UnixStream)>> = OnceLock::new();
static CONNECTION_TOKEN: AtomicUsize = AtomicUsize::new(usize::MAX);

impl ConnectionThread {
    pub fn spawn() -> JoinHandle<()> {
        static mut POLL: OnceLock<Poll> = OnceLock::new();
        unsafe { POLL.set(Poll::new().unwrap()) }.unwrap();
        REGISTRY
            .set(unsafe { POLL.get() }.unwrap().registry())
            .unwrap();
        unsafe { CONNECTIONS.set(HashMap::new()) }.unwrap();

        thread::spawn(|| {
            let processor = ProcessorThread::sender();

            let poll = unsafe { POLL.get_mut() }.unwrap();
            let mut events = Events::with_capacity(1024);

            loop {
                if let Err(err) = poll.poll(&mut events, None) {
                    if err.kind() == io::ErrorKind::Interrupted {
                        continue;
                    } else {
                        Err::<(), io::Error>(err).unwrap();
                        unreachable!()
                    }
                }

                for event in events.iter() {
                    match event.token() {
                        server if Connection::connections().contains_key(&event.token().0) => {
                            loop {
                                let connection_entry = Connection::get_mut(&server.0).unwrap();
                                let (mut connection, _address) =
                                    match connection_entry.server.accept() {
                                        Ok(res) => res,
                                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                            break;
                                        }
                                        e => {
                                            e.unwrap();
                                            unreachable!()
                                        }
                                    };

                                println!("Accepted connection from: {:?}", _address);

                                let token = CONNECTION_TOKEN
                                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                                connection_entry.connections.insert(token);
                                REGISTRY
                                    .get()
                                    .unwrap()
                                    .register(
                                        &mut connection,
                                        Token(token),
                                        Interest::READABLE.add(Interest::WRITABLE),
                                    )
                                    .unwrap();
                                unsafe { CONNECTIONS.get_mut() }
                                    .unwrap()
                                    .insert(token, (server.0, connection));
                            }
                        }
                        token => {
                            if let Some((server, connection)) =
                                unsafe { CONNECTIONS.get_mut() }.unwrap().get_mut(&token.0)
                            {
                                let done = handle_event(connection, event, *server, &processor).unwrap();
                                if done {
                                    REGISTRY.get().unwrap().deregister(connection).unwrap();
                                    unsafe { CONNECTIONS.get_mut() }
                                        .unwrap()
                                        .remove(&token.0);
                                    Connection::get_mut(&server).unwrap().connections.remove(&token.0);
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

fn handle_event(connection: &mut UnixStream, event: &Event, server: usize, sender: &Sender<ProcessorEvent>) -> io::Result<bool> {
    if event.is_readable() {
        let mut recieved_data = Vec::new();

        let _ = default_read_to_end(connection, &mut recieved_data, None);

        if !recieved_data.is_empty() {
            sender.send(ProcessorEvent::Packet { source: server, data: recieved_data }).unwrap();
            return Ok(false)
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
