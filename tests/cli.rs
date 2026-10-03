use std::io::Write;
use std::process::{Command, Stdio};

fn heck(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_heck"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn prints_only_complete_candidates_on_stdout() {
    let output = heck(&["suggest", "--", "cargo bulid --release"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "cargo build --release\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn stdin_preserves_trailing_spaces() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_heck"))
        .args(["suggest", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"git stats --short  ")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "git status --short  \ngit stage --short  \ngit stash --short  \n"
    );
}

#[test]
fn distinguishes_no_match_and_unsupported_input() {
    let no_match = heck(&["suggest", "--", "git status"]);
    assert_eq!(no_match.status.code(), Some(1));
    assert!(no_match.stdout.is_empty());
    let unsupported = heck(&["suggest", "--", "git stats | cat"]);
    assert_eq!(unsupported.status.code(), Some(2));
    assert!(unsupported.stdout.is_empty());
}

#[test]
fn malformed_cli_input_is_rejected() {
    for args in [
        vec!["suggest"],
        vec!["suggest", "--stdin", "--", "git stats"],
        vec!["suggest", "--", "git", "stats"],
        vec!["suggest", "--unknown", "--", "git stats"],
        vec!["suggest", "--"],
    ] {
        let output = heck(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}
