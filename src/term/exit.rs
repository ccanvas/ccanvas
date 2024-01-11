use nix::sys::signal::{self, SigHandler, Signal};

use crate::values::{ROOT, SCREEN};
use std::{fs, io::Write};

/// run when exiting
pub async fn exit() {
    write!(
        unsafe { SCREEN.get().unwrap() }.lock().unwrap(),
        "{}{}{}",
        termion::cursor::Show,
        termion::cursor::Restore,
        termion::screen::ToMainScreen,
    )
    .unwrap();

    // changes the sig handler back to default
    unsafe {
        signal::sigaction(
            Signal::SIGWINCH,
            &signal::SigAction::new(
                SigHandler::SigDfl,
                signal::SaFlags::empty(),
                signal::SigSet::empty(),
            ),
        )
        .unwrap();
    }

    // drop screen so the term actually gets restored
    unsafe { SCREEN.take() };
    fs::remove_dir_all(ROOT.get().unwrap()).unwrap();
}
