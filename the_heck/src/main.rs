mod shell_history;
mod rules;


fn main() {
    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    let last_command = shell_history::read_last_line_history_file(hist_path);
    // TODO: why is this backwards?
    let program_name = &last_command[1];
    let program_command = &last_command[0];

    // Search for known program in last command
    the_heck::search_known_programs(&program_name);

    // Search for possible correct commands

    // Fix command
    // let check_command = rules::match_command(last_command);
    let string_to_test = "sl".to_string();
    let check_command = rules::fix_ls(string_to_test);
    println!("Fixed command: {:?}", check_command);

    // Return fix to shell
}
