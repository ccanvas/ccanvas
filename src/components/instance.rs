use std::{
    fs, panic,
    path::{Path, PathBuf},
    process,
    sync::OnceLock,
};

use crate::{ConnectionThread, MessageThread, ProcessorThread};

use super::connection::Connection;

static INSTANCE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub struct Instance;

impl Instance {
    pub fn init() {
        Self::panic();

        let path =
            PathBuf::from(std::env::var("CCANVAS_PATH").expect("CCANVAS_PATH not specified"));

        INSTANCE_PATH.set(path).unwrap();

        ProcessorThread::spawn();
        ConnectionThread::spawn();
        MessageThread::spawn();
        Self::path_create();
        Connection::init();
    }

    pub fn panic() {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            default(info);
            process::exit(1);
        }));
    }

    pub fn path() -> &'static Path {
        INSTANCE_PATH.get().unwrap()
    }

    pub fn path_create() -> &'static Path {
        let path = Self::path();
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }
        path
    }

    pub fn conn_path(id: usize) -> PathBuf {
        Self::path().join(id.to_string())
    }

    pub fn conn_path_create(id: usize) -> PathBuf {
        let path = Self::conn_path(id);
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }

        path
    }

    pub fn conn_client_sock(id: usize) -> PathBuf {
        Self::conn_path(id).join("client.sock")
    }

    pub fn conn_server_sock(id: usize) -> PathBuf {
        Self::conn_path(id).join("server.sock")
    }
}
