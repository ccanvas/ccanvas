use std::process;

pub fn commands() -> Vec<Vec<String>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut commands = Vec::new();
    let mut command = Vec::new();

    for arg in args.into_iter() {
        if arg == "$" {
            if command.len() < 2 {
                println!("Bad arguments: expect `ccanvas [label] [command] (args..)`");
                process::exit(-1);
            }
            commands.push(std::mem::take(&mut command));
            continue;
        }

        command.push(arg)
    }

    if !command.is_empty() {
        if command.len() < 2 {
            println!("Bad arguments: expect `ccanvas [label] [command] (args..)`");
            process::exit(-1);
        }
        commands.push(std::mem::take(&mut command));
    }

    if commands.is_empty() {
        println!("Bad arguments: expect `ccanvas [label] [command] (args..)`");
        process::exit(-1);
    }

    commands
}
