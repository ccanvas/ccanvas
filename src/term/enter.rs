use std::{
    fs,
    io::{stdout, Write},
    path::PathBuf,
    process,
    sync::Mutex,
};

use termion::{input::MouseTerminal, raw::IntoRawMode, screen::IntoAlternateScreen};

use crate::{
    structs::{Discriminator, Storage},
    values::{FOCUSED, ROOT, SCREEN},
};

/// run when entering
pub async fn init() {
    let root = PathBuf::from("/tmp")
        .join("ccanvas")
        .join(process::id().to_string());

    Storage::remove_if_exist(&root).await.unwrap();

    fs::create_dir_all(&root).unwrap();
    ROOT.set(root).unwrap();

    #[cfg(feature = "log")]
    {
        let log_file = dirs::data_dir().unwrap().join("ccanvas.log");
        simplelog::WriteLogger::init(
            log::LevelFilter::Trace,
            simplelog::ConfigBuilder::new()
                .set_max_level(log::LevelFilter::Trace)
                .set_location_level(log::LevelFilter::Trace)
                .build(),
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(log_file)
                .unwrap(),
        )
        .unwrap();
    }
}

pub fn enter() {
    let mut screen = MouseTerminal::from(
        stdout()
            .into_raw_mode()
            .unwrap()
            .into_alternate_screen()
            .unwrap(),
    );
    write!(screen, "{}", termion::clear::All).unwrap();
    screen.flush().unwrap();
    FOCUSED.set(Mutex::new(Discriminator::master())).unwrap();
    let _ = SCREEN.set(Mutex::new(Some(screen)));
}
