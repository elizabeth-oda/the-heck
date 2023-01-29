pub fn fix_git() {
    println!("Hello from Fix Git!")
}

pub fn fix_ls(last_command: String) -> String {
    println!("Command to fix: {}", last_command);
    let mut correct_command: &str = "Empty";
    if last_command == "sl" {
        correct_command = "ls";
    } else {
        correct_command = "Unknown string.";
    }

    correct_command.to_string()
}
