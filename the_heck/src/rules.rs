pub fn match_command(last_command: String) -> String {
    // There is only one possibility, 'sl'
    println!("Command to fix: {}", last_command);
    let mut correct_command: &str = "Empty";
    if last_command == "sl" {
        correct_command = "ls";
    } else {
        correct_command = "Unknown string.";
    }

    correct_command.to_string()
}