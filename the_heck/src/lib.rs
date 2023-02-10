mod rules;

pub fn fix_from_known_programs(split_last_command: Vec<String>) -> Vec<String>{
    // Checks whether the command contains calls a program known to the-heck
    let program_name = &split_last_command[0];
    let program_name = program_name.to_owned();
    println!("Program name: {}", program_name);

    let wrong_command = &split_last_command[1];
    let wrong_command = wrong_command.to_owned();
    println!("Wrong command: {}", wrong_command);

    let fixed_command = match program_name.as_ref() {
        "git" => rules::fix_git(&wrong_command),
        "sl" => rules::fix_ls(&wrong_command),
        "cargo" => rules::fix_cargo(&wrong_command),
        _ => rules::not_implemented(&wrong_command),
    };
    
    fixed_command.expect("Failed to return fixed command as a string.")

}

pub fn search_unknown_program_name() {

}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}