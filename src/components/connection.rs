use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::OnceLock,
};

use mio::net::UnixStream;

static mut CONNECTIONS: OnceLock<HashMap<usize, Connection>> = OnceLock::new();
static mut LABEL_TO_ID: OnceLock<HashMap<String, usize>> = OnceLock::new();

#[derive(Debug)]
pub struct Connection {
    // message | message
    pub client: Option<UnixStream>,
    // connection | connection
    pub connections: HashSet<usize>,
}

impl Connection {
    pub fn init() {
        unsafe { CONNECTIONS.set(HashMap::new()) }.unwrap();
        unsafe { LABEL_TO_ID.set(HashMap::new()) }.unwrap();
        Self::create(0, &None, "master".to_string());
    }

    pub fn create(id: usize, path: &Option<String>, label: String) -> bool {
        let conns = Self::connections_mut();

        if conns.contains_key(&id) {
            return false;
        }

        match unsafe { LABEL_TO_ID.get_mut() }.unwrap().entry(label) {
            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(id),
            _ => return false,
        };

        let client = if let Some(path) = path {
            UnixStream::connect(path).ok()
        } else {
            None
        };

        let entry = Self {
            client,
            connections: HashSet::new(),
        };

        conns.insert(id, entry);

        true
    }
}

impl Connection {
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
        if self
            .client
            .as_mut()
            .is_some_and(|stream| stream.write_all(bytes).is_err())
        {
            self.client = None;
            // TODO maybe unsubscribe from everything etc
        }
    }
}
