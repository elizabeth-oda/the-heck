use rev_buf_reader::RevBufReader;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn get_current_shell_config() -> PathBuf {
    let shell_path = env::var("SHELL").expect("Cannot find current shell");
    let mut user_home_dir = home::home_dir().expect("No home directory found.");

    // TODO: user might have changed their default shell config.
    // How to handle that?
    match shell_path.split("/").last().unwrap() {
        "zsh" => user_home_dir.push(".zshrc"),
        "bash" => user_home_dir.push(".bashrc"),
        _ => panic!("Support for shell '{}' is not implemented yet.", shell_path),
    };
    user_home_dir
}

fn get_history_file_path_from_config(config_file_path: &PathBuf) -> Option<PathBuf> {
    let zfile = File::open(config_file_path).expect("Could not open shell config file.");
    let reader = BufReader::new(zfile);
    let line_marker = "HISTFILE=";

    let mut lines_iter = reader.lines().filter_map(Result::ok);

    while let Some(line) = lines_iter.next() {
        if line.starts_with(line_marker) {
            let temp = PathBuf::from(line.split(line_marker).last().unwrap());
            let mut hist_file_path = PathBuf::new();
            if temp.starts_with("~") {
                if let Some(home_dir) = home::home_dir() {
                    hist_file_path.push(home_dir);
                    hist_file_path.push(temp.strip_prefix("~").unwrap());
                }
            } else {
                hist_file_path.push(temp);
            }
            return Some(hist_file_path);
        }
    }

    None
}

pub fn get_history_file_path() -> PathBuf {
    let shell_config_path = get_current_shell_config();

    if let Some(hist_path_from_config) = get_history_file_path_from_config(&shell_config_path) {
        // get history file from shell config if configured there
        hist_path_from_config
    } else {
        // otherwise, use default history file for given shell
        let mut hist_path_default = shell_config_path.parent().unwrap().to_owned().clone();
        match shell_config_path.file_name().unwrap().to_str() {
            Some(".zshrc") => hist_path_default.push(".zsh_history"),
            Some(".bashrc") => hist_path_default.push(".bash_history"),
            _ => panic!(
                "Config file {} is not supported yet.",
                shell_config_path.to_str().unwrap()
            ),
        }
        hist_path_default
    }
}

pub fn read_last_line_history_file(hist_file_path: PathBuf) -> String {
    let file = File::open(hist_file_path).expect("Could not open history file.");
    let buf = RevBufReader::new(file);
    // Takes the last 128 bytes of the file
    let last_lines_in_file: Vec<String> = buf
        .lines()
        .take(128)
        .map(|l| l.expect("Could not parse line."))
        .collect();
    // Splits the last line at the semicolon which separates the command from the timestamp
    let last_line: Vec<String> = last_lines_in_file[1]
        .split(';')
        .map(|borrow| borrow.to_owned())
        .collect();
    let last_command = last_line[1].to_string();
    // println!("Last command from shell: {}", last_command);

    last_command
}
