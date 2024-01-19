use std::{env, sync::Arc, time::Duration};

use ccanvas::{
    structs::Space,
    term::{enter, exit, init},
};
use tokio::runtime::Runtime;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut commands = Vec::new();
    let mut command = Vec::new();

    for arg in args.iter() {

        if arg == "$$" {
            if command.len() < 2 {
                println!("Bad arguments: expect `ccanvas [label] [command] (args..)`");
                return;
            }
            commands.push(std::mem::take(&mut command));
            continue;
        }
        
        command.push(arg)
    }

    if !command.is_empty() {
        if command.len() < 2 {
            println!("Bad arguments: expect `ccanvas [label] [command] (args..)`");
            return;
        }
        commands.push(std::mem::take(&mut command));
    }

    let runtime = Runtime::new().unwrap();

    runtime.block_on(init());

    // creates new master space
    let master = Arc::new(runtime.block_on(Space::new("master".to_string())));
    let handle = runtime.spawn(Space::listen(master.clone()));
    if let Err(e) =
        runtime.block_on(master.spawn(args[0].clone(), args[1].clone(), args[2..].to_vec()))
    {
        // there is no reason to continue if spawning the component failed
        // might be a type, or whatever
        eprintln!("{e}");
    } else {
        enter();
        runtime.block_on(handle).unwrap();
        runtime.block_on(exit());
    }

    // get rid of everyting, kills all processes, etc
    runtime.shutdown_timeout(Duration::from_secs(0));
}
