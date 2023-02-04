mod rules;

pub fn search_known_programs(program_name: String) -> String{
    // Checks whether the command contains calls a program known to the-heck
    println!("Program name: {}", program_name);
    let fixed_command = match program_name.as_ref() {
        "git" => rules::fix_git(),
        "sl" => rules::fix_ls(),
        "cargo" => rules::fix_cargo(),
        _ => ("Not implemented yet.").to_string(),
    };
    
    found_program_name.to_string()

}

pub fn search_unknown_program_name() {

}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}