mod shell_history;

fn main() {
    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    let last_command = shell_history::read_last_line_history_file(hist_path);
    // Split the last command into words
    let split_last_command: Vec<&str> = last_command.split(' ').collect();
    println!("Command line arguments: {:?}", split_last_command);

    let program_name: &str = split_last_command[0];
    let wrong_command: &str = split_last_command[1];
    println!("Wrong command: {}", wrong_command);

    let program_commands = the_heck::check_known_programs(split_last_command);
    // Attempt to fix command if program name is correct
    let mut fixed_command = vec!["If you see this, that's bad".to_string()];

    // TODO: Use if let instead of this mess
    if program_commands
        .iter()
        .any(|&i| i == "Not implemented!")
    {
        // If the program name is unknown, fuzzy search the program name
        let fixed_program_name = the_heck::fix_program_name(program_name).unwrap();
        let fixed_program_name = fixed_program_name.iter().map(|s| s.as_str()).collect();
        let new_program_commands = the_heck::check_known_programs(fixed_program_name);
        // Fix the command using the new program name
        fixed_command = the_heck::fix_command(wrong_command, new_program_commands).unwrap();
    } else {
        fixed_command = the_heck::fix_command(wrong_command, program_commands).unwrap();
    };

    println!("Fixed command: {:?}", fixed_command);

    // Return fix to shell
}
