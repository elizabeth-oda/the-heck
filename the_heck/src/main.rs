use std::env;
use std::process;
use the_heck::UserInput;

fn main() {
    // Parse arguments
    let args: Vec<String> = env::args().collect();

    let user_input = UserInput::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // println!("{0}", user_input.wrong_command);
    // println!("{0}", user_input.options);
    
    // Check whether arg is in known args
    // Fix command
    // Return fix to shell
}
