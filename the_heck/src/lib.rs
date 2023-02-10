use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Set};

pub fn fix_from_known_programs(split_last_command: Vec<&str>) -> Vec<String> {
    // Checks whether the command contains calls a program known to the-heck
    let program_name: &str = split_last_command[0];
    println!("Program name: {}", program_name);

    let wrong_command: &str = split_last_command[1];
    println!("Wrong command: {}", wrong_command);

    let this_guy = fix_command(wrong_command, get_possible_commands(program_name)).unwrap();

    this_guy
}

pub fn fix_command(
    wrong_command: &str,
    possible_commands: &[&str],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let set = Set::from_iter(possible_commands)?;
    let lev = Levenshtein::new(wrong_command, 2)?;
    let stream = set.search(lev).into_stream();
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
