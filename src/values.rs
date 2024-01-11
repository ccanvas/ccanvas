use std::{io::Stdout, path::PathBuf, sync::Mutex};

use termion::{input::MouseTerminal, raw::RawTerminal, screen::AlternateScreen};
use tokio::sync::OnceCell;

use crate::structs::Discriminator;

pub static mut FOCUSED: OnceCell<Discriminator> = OnceCell::const_new();
pub static mut SCREEN: OnceCell<Mutex<MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>>> =
    OnceCell::const_new();
pub static ROOT: OnceCell<PathBuf> = OnceCell::const_new();
