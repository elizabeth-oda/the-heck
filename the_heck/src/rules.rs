use fst::{IntoStreamer, Streamer, Set};
use fst::automaton::Levenshtein;


pub fn fix_git() -> String {
    let git_keys = vec!["add", "rm", "restore", "restore --staged", "status"];
    let correct_command = "git";
    correct_command.to_string()
}

pub fn fix_ls() -> String {
    let correct_command = "ls";
    correct_command.to_string()
}

pub fn fix_cargo() -> String {
    let cargo_keys = vec!["run", "build", "test", "install", "uninstall"];
    let correct_command = "Hi, Cargo";
    correct_command.to_string()
}