use std::env;
use std::process;
use the_heck::UserInput;
mod shell_history;

fn main() {
    // Temporary arguments for testing
    let args: Vec<String> = env::args().collect();
    let user_input = UserInput::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    let last_line = shell_history::read_last_line_history_file(&hist_path);
    println!("{:?}", last_line);
    // Check whether arg is in known args
    // Fix command
    // Return fix to shell
}
