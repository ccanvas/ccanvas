use std::{io::Stdout, path::PathBuf, sync::Mutex};

use termion::{input::MouseTerminal, raw::RawTerminal, screen::AlternateScreen};
use tokio::sync::OnceCell;

use crate::structs::Discriminator;

type Term = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

pub static FOCUSED: OnceCell<Mutex<Discriminator>> = OnceCell::const_new();
pub static SCREEN: OnceCell<Mutex<Option<Term>>> = OnceCell::const_new();
pub static ROOT: OnceCell<PathBuf> = OnceCell::const_new();
