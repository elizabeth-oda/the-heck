use the_heck::{suggest, Context, Outcome};

fn corrections(input: &str) -> Vec<the_heck::Suggestion> {
    match suggest(input, &Context::default()) {
        Outcome::Suggestions(suggestions) => suggestions,
        other => panic!("Expected suggestions for {input:?}, got {other:?}"),
    }
}

#[test]
fn preserves_the_entire_command_except_the_edited_spans() {
    for (input, expected) in [
        ("gti", "git"),
        ("git stats --short", "git status --short"),
        ("gti status --short", "git status --short"),
        ("gti stats --short", "git status --short"),
        ("cargo  bulid --release", "cargo  build --release"),
        ("  cargo\tbulid  --release  ", "  cargo\tbuild  --release  "),
        ("git ad 'my file.txt'", "git add 'my file.txt'"),
        ("git ad \"日本語 file.txt\"", "git add \"日本語 file.txt\""),
        ("git ad my\\ file.txt", "git add my\\ file.txt"),
        (
            "git ad '$HOME; $(echo untouched)'",
            "git add '$HOME; $(echo untouched)'",
        ),
        ("git ad \"$HOME/file.txt\"", "git add \"$HOME/file.txt\""),
        (
            "cargo bulid --features 'a,b'",
            "cargo build --features 'a,b'",
        ),
        ("git restroe --staged .", "git restore --staged ."),
    ] {
        let suggestions = corrections(input);
        assert_eq!(suggestions[0].command, expected);
    }
}

#[test]
fn valid_and_unrecognized_commands_have_no_match() {
    for input in [
        "",
        "  \t",
        "git",
        "cargo",
        "git status --short",
        "git restore --staged .",
        "cargo test",
        "cargo b",
        "git stage .",
        "git log",
        "totally-unknown arg",
        "git zzzzzzzzzzzzzzzzzz",
        "sl",
    ] {
        assert_eq!(
            suggest(input, &Context::default()),
            Outcome::NoMatch,
            "{input:?}"
        );
    }
}

#[test]
fn shell_aliases_functions_and_existing_programs_are_preserved() {
    for input in ["gti status", "git stats", "cargo bulid"] {
        assert_eq!(
            suggest(
                input,
                &Context {
                    program_shadowed: true,
                    ..Context::default()
                }
            ),
            Outcome::NoMatch
        );
    }
    assert_eq!(
        suggest(
            "gti status",
            &Context {
                program_known: true,
                ..Context::default()
            }
        ),
        Outcome::NoMatch
    );
    assert_eq!(
        match suggest(
            "git stats",
            &Context {
                program_known: true,
                ..Context::default()
            }
        ) {
            Outcome::Suggestions(suggestions) => suggestions[0].command.clone(),
            other => panic!("{other:?}"),
        },
        "git status"
    );
}

#[test]
fn declines_syntax_it_cannot_preserve_confidently() {
    for input in [
        "git stats | cat",
        "git stats > out",
        "git stats && echo ok",
        "git stats; echo ok",
        "git stats\necho ok",
        "git stats\r",
        "git stats\0",
        "git stats $(touch marker)",
        "git stats `touch marker`",
        "git stats \"$(touch marker)\"",
        "git stats 'unfinished",
        "git stats \\",
        "git stats # comment",
        "FOO=bar git stats",
        "git -C somewhere stats",
        "cargo +nightly bulid",
        "'git' stats",
        "git \"stats\"",
        "git st\\ats",
        "$PROGRAM stats",
        "git stats\u{1b}[2J",
    ] {
        assert!(
            matches!(suggest(input, &Context::default()), Outcome::Unsupported(_)),
            "{input:?}"
        );
    }
}

#[test]
fn ambiguous_corrections_are_ranked() {
    let commands: Vec<_> = corrections("git stat --short")
        .into_iter()
        .map(|suggestion| suggestion.command)
        .collect();
    assert_eq!(
        commands,
        [
            "git stage --short",
            "git stash --short",
            "git status --short",
            "git tag --short"
        ]
    );
}

#[test]
fn oversized_input_is_declined() {
    assert!(matches!(
        suggest(&"a".repeat(65_537), &Context::default()),
        Outcome::Unsupported(_)
    ));
}
