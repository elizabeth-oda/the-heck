use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::io::BufRead;
use home;
use rev_buf_reader::RevBufReader;


pub fn get_history_file_path() -> PathBuf {
    let user_home_dir = home::home_dir().expect("No home directory found.");
    let mut root_dir = user_home_dir.as_path();
    let mut hist_file_path = PathBuf::from(root_dir);
    hist_file_path.push(".zsh_history");
    // println!("History file path: {}", hist_file_path.display());

    hist_file_path
}

pub fn read_full_history_file(hist_file_path: &PathBuf) -> String {
    let mut f = File::open(hist_file_path).unwrap_or_else(|_| panic!("File not found {:?}.", &hist_file_path));
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap_or_else(|_| panic!("Unable to read from {:?}.", &hist_file_path));
    String::from_utf8_lossy(&buffer).to_string()
}

pub fn read_last_line_history_file(hist_file_path: &PathBuf) -> Vec<String> {
    let file = File::open(hist_file_path).expect("Could not open file.");
    let mut buf = RevBufReader::new(file);
    buf.lines().take(256).map(|l| l.expect("Could not parse line")).collect()
}
