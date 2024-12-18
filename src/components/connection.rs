use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::PathBuf,
    sync::OnceLock,
};

use mio::{
    net::{UnixListener, UnixStream},
    Interest, Token,
};

use crate::ConnectionThread;

use super::Instance;

static mut CONNECTIONS: OnceLock<HashMap<usize, Connection>> = OnceLock::new();

#[derive(Debug)]
pub struct Connection {
    // read | write
    // processor | processor
    pub parent: usize,
    // processor | processor
    pub children: HashSet<usize>,

    // message | message
    pub client: Option<PathBuf>,
    // connection | none
    pub server: UnixListener,
    // connection | connection
    pub connections: HashSet<usize>,
}

impl Connection {
    pub fn init() {
        unsafe { CONNECTIONS.set(HashMap::new()) }.unwrap();
        Self::create(0, 0).unwrap();
    }

    pub fn create(id: usize, parent: usize) -> Result<(), &'static str> {
        let conns = Self::connections_mut();

        if conns.contains_key(&id) {
            return Err("id");
        }

        if id != parent {
            conns.get_mut(&parent).unwrap().children.insert(id);
        }

        Instance::conn_path_create(id);
        let server = UnixListener::bind(Instance::conn_server_sock(id));
        let client_path = Instance::conn_client_sock(id);

        if server.is_err() {
            return Err("socket");
        }

        let mut entry = Self {
            parent,
            children: HashSet::new(),
            client: client_path.exists().then_some(client_path),
            server: server.unwrap(),
            connections: HashSet::new(),
        };

        ConnectionThread::add_server(&mut entry.server, id);

        conns.insert(id, entry);

        Ok(())
    }
}

impl Connection {
    pub fn descendants(&'static self) -> Box<dyn Iterator<Item = usize>> {
        Box::new(self.children.iter().flat_map(|&child| {
            Self::connections()
                .get(&child)
                .unwrap()
                .descendants()
                .chain(std::iter::once(child))
        }))
    }

    pub fn connections() -> &'static HashMap<usize, Connection> {
        unsafe { CONNECTIONS.get() }.unwrap()
    }

    pub fn connections_mut() -> &'static mut HashMap<usize, Connection> {
        unsafe { CONNECTIONS.get_mut() }.unwrap()
    }

    pub fn get(id: &usize) -> Option<&'static Connection> {
        Self::connections().get(id)
    }

    pub fn get_mut(id: &usize) -> Option<&'static mut Connection> {
        Self::connections_mut().get_mut(id)
    }
}

impl Connection {
    pub fn write(&mut self, bytes: &[u8]) {
        if self.client.as_ref().is_some_and(|path| {
            UnixStream::connect(path)
                .and_then(|mut socket| socket.write_all(bytes))
                .is_err()
        }) {
            self.client = None;
        }
    }
}
