use dialoguer::{theme::ColorfulTheme, Select};

mod shell_history;

fn main() {
    // Get the last shell command
    let hist_path = shell_history::get_history_file_path();
    println!("History file: {}", hist_path.to_str().unwrap());
    let last_command = shell_history::get_last_command_from_shell_history(&hist_path);
    // Split the last command into words
    let split_last_command = last_command.split(' ').collect();
    // println!("Command line arguments: {:?}", split_last_command);

    // Correct the command
    let fixed_command = the_heck::correcter(split_last_command);
    // println!("Fixed command: {:?}", fixed_command);

    // Return fix to shell
    let _selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Is this command correct? [Press Enter] ")
        .default(0)
        .items(&fixed_command)
        .interact()
        .expect("Unable to generate correct command.");

    the_heck::push_command_to_cli(last_command, fixed_command);
}
