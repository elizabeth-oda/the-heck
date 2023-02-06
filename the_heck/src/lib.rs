mod rules;

pub fn fix_from_known_programs(split_last_command: Vec<String>) -> String{
    // Checks whether the command contains calls a program known to the-heck
    let mut program_name = &split_last_command[0];
    let program_name = program_name.to_owned();
    println!("Program name: {}", program_name);

    let fixed_command = match program_name.as_ref() {
        "git" => rules::fix_git(),
        "sl" => rules::fix_ls(),
        "cargo" => rules::fix_cargo(),
        _ => ("Not implemented yet.").to_string(),
    };
    
    fixed_command.to_string()

}

pub fn search_unknown_program_name() {

}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}