use std::env;
use std::process;
use the_heck::UserInput;
mod shell_history;

fn main() {
    // TODO: implement shell search
    // TEMP: Parse arguments
    let args: Vec<String> = env::args().collect();

    let user_input = UserInput::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let hist_path = shell_history::read_history_from_file();



    // println!("{0}", user_input.wrong_command);
    // println!("{0}", user_input.options);

    // Check whether arg is in known args
    // Fix command
    // Return fix to shell
}
