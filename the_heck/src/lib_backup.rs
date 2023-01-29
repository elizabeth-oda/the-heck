macro_rules! vec_of_strings {
    // Call String::from on each list element
    ($($str:expr),*) => ({
        vec![$(String::from($str),)*] as Vec<String>
    });
}

pub fn search_known_programs(last_command: String) {
     // Checks whether the command contains calls a program known to the-heck
     let known_programs = vec_of_strings![
        "git",
     ];

}

pub enum KnownProgram {
    git(String),

}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}