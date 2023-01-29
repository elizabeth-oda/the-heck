mod rules;

pub fn search_known_programs(last_command: Vec<&str>) {
    // Checks whether the command contains calls a program known to the-heck
    let first_arg = last_command[0].to_string();
    println!("First arg: {:?}", first_arg);
    match first_arg.as_ref() {
        "git" => rules::fix_git(),
        &_ => todo!(),
    }
}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}