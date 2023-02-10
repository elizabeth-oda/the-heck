use fst::{IntoStreamer, Set};
use fst::automaton::Levenshtein;


pub fn fix_git(wrong_command: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>  {
    let keys = vec!["add", "restore", "restore --staged", "rm", "status"];
    let set = Set::from_iter(keys)?;
    let lev = Levenshtein::new(wrong_command, 1)?;
    let stream = set.search(lev).into_stream();
    let keys = stream.into_strs()?;
    Ok(keys)
}

pub fn fix_ls(wrong_command: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let keys = vec!["ls"];
    let set = Set::from_iter(keys)?;
    let lev = Levenshtein::new(wrong_command, 1)?;
    let stream = set.search(lev).into_stream();
    let keys = stream.into_strs()?;
    Ok(keys)
}

pub fn fix_cargo(wrong_command: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let keys = vec!["build", "install", "run", "test", "uninstall"];
    let set = Set::from_iter(keys)?;
    let lev = Levenshtein::new(wrong_command, 1)?;
    let stream = set.search(lev).into_stream();
    let keys = stream.into_strs()?;
    Ok(keys)
}

pub fn not_implemented(wrong_command: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let unknown_command_msg = format!("{} is not implemented yet", wrong_command);
    let unknown_command = vec![unknown_command_msg];

    Ok(unknown_command)
}