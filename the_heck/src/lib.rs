use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set};

pub fn check_known_programs(split_last_command: Vec<&str>) -> &[&str] {
    // Checks whether the command contains calls a program known to the-heck
    let program_name: &str = split_last_command[0];
    println!("Program name: {}", program_name);
    let program_commands = get_possible_commands(program_name);

    program_commands
}

pub fn fix_command(
    wrong_command: &str,
    possible_commands: &[&str],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Implements a fuzzy search of possible commands against the query
    let set = Set::from_iter(possible_commands)?;
    // The maximum Levenshtein distance = 2
    let lev = Levenshtein::new(wrong_command, 2)?;
    let stream = set.search(lev).into_stream();
    // Returns the list of possible commands
    let keys = stream.into_strs()?;

    Ok(keys)
}

pub fn fix_program_name(wrong_program_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let known_programs = vec!["cargo", "git", "sl"];
    // Implements a fuzzy search of possible commands against the query
    let set = Set::from_iter(known_programs)?;
    // The maximum Levenshtein distance = 2
    let lev = Levenshtein::new(wrong_program_name, 2)?;
    let stream = set.search(lev).into_stream();
    // Returns the list of possible commands
    let keys = stream.into_strs()?;

    Ok(keys)
}


fn get_possible_commands(prog_name: &str) -> &'static [&'static str] {
    match prog_name {
        "git" => &["add", "restore", "restore --staged", "rm", "status"],
        "sl" => &["ls"],
        "cargo" => &[
            "build",
            "clippy",
            "fmt",
            "install",
            "run",
            "test",
            "uninstall",
        ],
        _ => &["Not implemented!"],
    }
}
