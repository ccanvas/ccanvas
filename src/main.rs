use std::sync::mpsc;

use ccanvas::Instance;

fn main() {
    Instance::init();

    let c = mpsc::channel::<()>();
    c.1.recv().unwrap();
}
