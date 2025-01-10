use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::PathBuf,
    sync::OnceLock,
};

use ccanvas_bindings::packets::RejConn;
use mio::net::UnixStream;

static mut CONNECTIONS: OnceLock<HashMap<usize, Connection>> = OnceLock::new();
static mut LABEL_TO_ID: OnceLock<HashMap<String, usize>> = OnceLock::new();

#[derive(Debug)]
pub struct Connection {
    // read | write
    // processor | processor
    pub parent: usize,
    // processor | processor
    pub children: HashSet<usize>,

    // message | message
    pub client: Option<UnixStream>,
    // connection | connection
    pub connections: HashSet<usize>,
}

impl Connection {
    pub fn init() {
        unsafe { CONNECTIONS.set(HashMap::new()) }.unwrap();
        unsafe { LABEL_TO_ID.set(HashMap::new()) }.unwrap();
        Self::create(0, &None, "", "master".to_string()).unwrap();
    }

    pub fn create(id: usize, path: &Option<PathBuf>, parent: &str, label: String) -> Result<(), RejConn> {
        let conns = Self::connections_mut();

        if conns.contains_key(&id) {
            return Err(RejConn::Id);
        }

        let parent_id = match parent {
            _ if parent.is_empty() => 0,
            _ if parent == &label => id,
            _ => if let Some(parent_id) = unsafe { LABEL_TO_ID.get() }.unwrap().get(parent) {
                *parent_id
            } else {
                return Err(RejConn::Parent)
            }
        };

        let client = if let Some(path) = path {
            UnixStream::connect(path).ok()
        } else {
            None
        };

        let entry = Self {
            parent: parent_id,
            children: HashSet::new(),
            client,
            connections: HashSet::new(),
        };

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
