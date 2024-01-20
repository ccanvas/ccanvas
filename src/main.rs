use std::{sync::Arc, time::Duration};

use ccanvas::{
    structs::Space,
    term::{commands, enter, exit, init},
};
use tokio::runtime::Runtime;

fn main() {
    let commands = commands();

    let runtime = Runtime::new().unwrap();

    runtime.block_on(init());

    // creates new master space
    let master = Arc::new(runtime.block_on(Space::new("master".to_string())));
    let handle = runtime.spawn(Space::listen(master.clone()));

    for command in commands {
        if let Err(e) = runtime.block_on(master.spawn(
            command[0].clone(),
            command[1].clone(),
            command[2..].to_vec(),
        )) {
            // there is no reason to continue if spawning the component failed
            // might be a type, or whatever
            eprintln!("{e}");
            runtime.shutdown_timeout(Duration::from_secs(0));
            return;
        }
    }

    enter();
    runtime.block_on(handle).unwrap();
    runtime.block_on(exit());

    // get rid of everyting, kills all processes, etc
    runtime.shutdown_timeout(Duration::from_secs(0));
}
