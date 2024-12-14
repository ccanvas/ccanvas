use std::{
    ffi::c_void,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use libc::{size_t, syscall};

use crate::{ConnectionThread, MessageThread};

use super::connection::Connection;

const GETRANDOM: i64 = 318;

static INSTANCE_ID: OnceLock<usize> = OnceLock::new();
static INSTANCE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub struct Instance;

impl Instance {
    pub fn init() {
        INSTANCE_ID.set(Self::get_random()).unwrap();
        INSTANCE_PATH
            .set(
                PathBuf::from(std::env::var("CCANVAS_PATH").unwrap_or("/tmp/ccanvas/".to_string()))
                    .join(INSTANCE_ID.get().unwrap().to_string()),
            )
            .unwrap();

        ConnectionThread::spawn();
        MessageThread::spawn();
        Self::path_create();
        Connection::init();
    }

    pub fn path() -> &'static Path {
        INSTANCE_PATH.get().unwrap()
    }

    pub fn path_create() -> &'static Path {
        let path = Self::path();
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
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

    pub fn get_random() -> usize {
        let mut buffer = [0u8; std::mem::size_of::<usize>()];
        let size = buffer.len() as size_t;
        let _ = unsafe { syscall(GETRANDOM, buffer.as_mut_ptr() as *mut c_void, size, 0) };
        usize::from_ne_bytes(buffer)
    }
}
