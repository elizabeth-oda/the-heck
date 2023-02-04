mod shell_history;
mod rules;


fn main() {
    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    let last_command = shell_history::read_last_line_history_file(hist_path);
    let split_last_command: Vec<String> = last_command.split(" ").map(|s| s.to_string()).collect();
    println!("{:?}", split_last_command);
    let mut program_name = &split_last_command[0];
    let program_name = program_name.to_owned();

    // Search for known program in last command
    the_heck::search_known_programs(program_name);

    // Search for possible correct commands

    // Fix command

    // Return fix to shell
}
