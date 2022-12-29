use std::error::Error;

pub struct UserInput {
    pub wrong_command: String,
    pub options: String,
}

impl UserInput {
    pub fn build(args: &[String]) -> Result<UserInput, &'static str> {
        if args.len () < 3 {
            return Err("Not enough arguments");
        }
        let wrong_command = args[1].clone();
        let options = args[2].clone();
    
        Ok(UserInput {wrong_command, options})
    }
}

pub fn fix_command() {
    // Fixes the previous command when 'theheck' is called.
}

pub fn get_raw_command () {
    // Gets the raw command from the command line.
}

pub fn check_known_args () {
    // Checks whether the argument is known and can be handled by theheck.
}

pub fn return_fixed_command () {
    // Returns the fixed command to the shell.
}