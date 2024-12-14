use std::{
    collections::{HashMap, HashSet},
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
    pub data: HashMap<Vec<u8>, Vec<u8>>,
    // processor | processor
    pub children: HashSet<usize>,

    // message | message
    pub client: Option<UnixStream>,
    // connection | none
    pub server: UnixListener,
}

impl Connection {
    pub fn init() {
        unsafe { CONNECTIONS.set(HashMap::new()) }.unwrap();
        Self::new(0, 0, HashMap::new()).unwrap();
    }

    pub fn new(
        id: usize,
        parent: usize,
        data: HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<(), &'static str> {
        let conns = Self::connections_mut();

        if conns.contains_key(&id) {
            return Err("id");
        }

        if id != parent {
            conns.get_mut(&parent).unwrap().children.insert(id);
        }

        Instance::conn_path_create(id);
        let server = UnixListener::bind(Instance::conn_server_sock(id));
        let client = UnixStream::connect(Instance::conn_client_sock(id));

        if server.is_err() {
            return Err("socket");
        }

        conns.insert(
            id,
            Self {
                parent,
                children: HashSet::new(),
                data,
                client: client.ok(),
                server: server.unwrap(),
            },
        );

        let entry = conns.get_mut(&id).unwrap();

        ConnectionThread::registry()
            .register(&mut entry.server, Token(id), Interest::READABLE)
            .unwrap();

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