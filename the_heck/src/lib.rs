mod rules;

pub fn search_known_programs(program_name: &String) {
    // Checks whether the command contains calls a program known to the-heck
    println!("Program name: {}", program_name);
    match program_name.as_ref() {
        "git" => rules::fix_git(),
        &_ => println!("Not implemented yet."),
    }
}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}