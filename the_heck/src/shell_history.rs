use std::path::PathBuf;
use home;

pub fn read_history_from_file() -> PathBuf {
    let user_home_dir = home::home_dir().expect("No home directory found.");
    let mut root_dir = user_home_dir.as_path();
    let mut hist_file_path = PathBuf::from(root_dir);
    hist_file_path.push(".zsh_history");
    println!("History file path: {}", hist_file_path.display());

    hist_file_path
}

