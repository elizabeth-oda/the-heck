mod shell_history;
mod rules;

fn main() {
    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    let last_command = shell_history::read_last_line_history_file(hist_path);
    // Split the last command into words
    let split_last_command: Vec<String> = last_command.split(' ').map(|s| s.to_string()).collect();
    println!("Command line arguments: {:?}", split_last_command);
    // Preserve the first argument, aka the program name
    let program_name = &split_last_command[0];
    let program_name = program_name.to_owned();
    // Attempt to fix command if program name is correct
    let fixed_command = the_heck::fix_from_known_programs(split_last_command);
    println!("Fixed command: {:?}", fixed_command);

    // Return fix to shell
}
