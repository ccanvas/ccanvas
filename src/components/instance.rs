use std::{
    fs, panic, path::{Path, PathBuf}, process, sync::OnceLock
};

use crate::{ConnectionThread, MessageThread, ProcessorThread};

use super::connection::Connection;

static INSTANCE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub struct Instance;

impl Instance {
    pub fn init() {
        Self::panic();

        let path =
            PathBuf::from(std::env::var("CCANVAS_SOCK").expect("CCANVAS_SOCK not specified"));

        INSTANCE_PATH.set(path).unwrap();

        ProcessorThread::spawn();
        ConnectionThread::spawn();
        MessageThread::spawn();
        Connection::init();
    }

    pub fn panic() {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            default(info);
            process::exit(1);
        }));
    }

    pub fn sock() -> &'static Path {
        INSTANCE_PATH.get().unwrap()
    }

    pub fn sock_create() -> &'static Path {
        let path = Self::sock();
        let parent = path.parent().unwrap_or(Path::new("/"));
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
        path
    }
}
