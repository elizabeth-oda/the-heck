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
    for input in ["gti status", "git stats", "cargo bulid", "uv sycn"] {
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
    for input in ["gti status", "vu sync"] {
        assert_eq!(
            suggest(
                input,
                &Context {
                    program_known: true,
                    ..Context::default()
                }
            ),
            Outcome::NoMatch
        );
    }
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

#[test]
fn nested_commands_and_prefix_options_preserve_arguments() {
    for (input, expected) in [
        ("git remote ad origin URL", "git remote add origin URL"),
        ("git remtoe add origin URL", "git remote add origin URL"),
        ("git remote -v shwo origin", "git remote -v show origin"),
        ("git stash psuh -m \"wip\"", "git stash push -m \"wip\""),
        ("git worktree lsit", "git worktree list"),
        (
            "git submodule --quiet udpate --init",
            "git submodule --quiet update --init",
        ),
        ("git bisect strat", "git bisect start"),
        ("git sparse-checkout reaply", "git sparse-checkout reapply"),
        ("git -C \"my repo\" stats", "git -C \"my repo\" status"),
        (
            "git -Cfirst -C second --no-pager stats",
            "git -Cfirst -C second --no-pager status",
        ),
        (
            "git --git-dir='my repo/.git' --work-tree \"my repo\" stats",
            "git --git-dir='my repo/.git' --work-tree \"my repo\" status",
        ),
        (
            "git -c core.pager=cat stats",
            "git -c core.pager=cat status",
        ),
        (
            "gh pr ceate --title \"Fix login\"",
            "gh pr create --title \"Fix login\"",
        ),
        (
            "gh pr --repo owner/repo veiw 42",
            "gh pr --repo owner/repo view 42",
        ),
        ("gh -Rowner/repo pr veiw 42", "gh -Rowner/repo pr view 42"),
        (
            "gh --repo=owner/repo pr veiw 42",
            "gh --repo=owner/repo pr view 42",
        ),
        (
            "gh pr --repo \"$REPO\" veiw 42",
            "gh pr --repo \"$REPO\" view 42",
        ),
        (
            "gh repo autolink ceate KEY URL",
            "gh repo autolink create KEY URL",
        ),
        (
            "gh ext isntall owner/gh-example",
            "gh ext install owner/gh-example",
        ),
        (
            "gh cs ports forwad 8080:8080",
            "gh cs ports forward 8080:8080",
        ),
        ("gh workflow veiw build.yml", "gh workflow view build.yml"),
    ] {
        assert_eq!(corrections(input)[0].command, expected, "{input}");
    }
}

#[test]
fn stack_support_requires_the_extension_and_preserves_its_arguments() {
    let aliased = Context {
        gh_stack: true,
        gh_aliases: vec!["stack".into()],
        ..Context::default()
    };
    assert_eq!(suggest("gh stakc view", &aliased), Outcome::NoMatch);

    let context = Context {
        gh_stack: true,
        gh_extensions: vec!["stack".into()],
        ..Context::default()
    };
    for (input, expected) in [
        ("gh stakc view", "gh stack view"),
        ("gh stack subimt --open", "gh stack submit --open"),
        ("gh stack rebsae --continue", "gh stack rebase --continue"),
        (
            "gh stack ad -Am \"日本語 message\" feature/login",
            "gh stack add -Am \"日本語 message\" feature/login",
        ),
        ("gh stack chekout 42", "gh stack checkout 42"),
        ("gh stack botom", "gh stack bottom"),
        ("gh stack unstakc --local", "gh stack unstack --local"),
    ] {
        let Outcome::Suggestions(suggestions) = suggest(input, &context) else {
            panic!("{input}");
        };
        assert_eq!(suggestions[0].command, expected);
    }
    for input in [
        "gh stack delete --local",
        "gh stack add -Am \"subimt\"",
        "gh stack checkout rebsae",
    ] {
        assert_eq!(suggest(input, &context), Outcome::NoMatch, "{input}");
    }
}

#[test]
fn command_arguments_and_configured_names_are_opaque() {
    let context = Context {
        git_aliases: vec!["stats".into()],
        gh_aliases: vec!["rp".into()],
        gh_extensions: vec!["stacks".into(), "stack".into()],
        ..Context::default()
    };
    for input in [
        "git stats --short",
        "gh rp ceate",
        "gh stacks subimt",
        "gh stack subimt",
        "git checkout psuh",
        "git remote add psuh URL",
        "git bisect good bda",
        "git stash -m psuh",
        "git stash -- psuh",
        "git submodule -- psuh",
        "gh pr view ceate",
        "gh pr create --title ceate",
        "gh api pr",
        "gh pr -- ceate",
        "gh pr ls",
        "gh pr co 42",
        "gh ext ls",
        "gh issue new",
        "gh repo deploy-key ls",
        "git -C somewhere -- stats",
    ] {
        assert_eq!(suggest(input, &context), Outcome::NoMatch, "{input}");
    }
}

#[test]
fn uncertain_options_and_dynamic_git_context_are_declined() {
    for input in [
        "git --unknown value stats",
        "git -C",
        "git -C \"$REPO\" stats",
        "git -C ~/repo stats",
        "git -C repos/* stats",
        "git -C {one,two} stats",
        "git -c include.path=$FILE stats",
        "gh pr --unknown value veiw",
        "gh --repo",
        "git remote --verbose=yes ad",
        "uv --unknown value sycn",
        "uv --project",
        "uv --offline=yes sycn",
        "uv -C repo sycn",
        "uv -p python3 sycn",
    ] {
        assert!(
            matches!(suggest(input, &Context::default()), Outcome::Unsupported(_)),
            "{input}"
        );
    }
}

#[test]
fn uv_commands_and_global_options_preserve_arguments() {
    for (input, expected) in [
        ("vu sync", "uv sync"),
        ("uv sncy", "uv sync"),
        (
            "uv pip isntall 'requests[socks]>=2'",
            "uv pip install 'requests[socks]>=2'",
        ),
        (
            "uv tool isntall --from 'ruff==0.14.0' ruff",
            "uv tool install --from 'ruff==0.14.0' ruff",
        ),
        ("uv pythno isntall 3.13", "uv python install 3.13"),
        ("uv cache claen ruff", "uv cache clean ruff"),
        (
            "uv auth logni https://packages.example.com",
            "uv auth login https://packages.example.com",
        ),
        ("uv self udpate", "uv self update"),
        (
            "  uv --project \"日本語 project\" --offline sycn  ",
            "  uv --project \"日本語 project\" --offline sync  ",
        ),
        (
            "uv --directory='./my project' pip --offline isntall numpy",
            "uv --directory='./my project' pip --offline install numpy",
        ),
        (
            "uv pip --project \"$PROJECT\" isntall numpy",
            "uv pip --project \"$PROJECT\" install numpy",
        ),
        ("uv -vvn tool isntall ruff", "uv -vvn tool install ruff"),
        (
            "uv --color always python --offline isntall 3.13",
            "uv --color always python --offline install 3.13",
        ),
        (
            "uv --cache-dir cache tool --project=. isntall ruff",
            "uv --cache-dir cache tool --project=. install ruff",
        ),
        (
            "uv --no-python-downloads python isntall 3.13",
            "uv --no-python-downloads python install 3.13",
        ),
    ] {
        assert_eq!(corrections(input)[0].command, expected, "{input}");
    }
}

#[test]
fn uv_package_names_and_executed_commands_are_opaque() {
    for input in [
        "uv",
        "uv sync",
        "uv add isntall",
        "uv pip install sncy",
        "uv python install isntall",
        "uv tool install rnu",
        "uv tool update --all",
        "uv python ls",
        "uv virtualenv sycn",
        "uv run pip isntall requests",
        "uv run --python 3.13 sycn --verbose",
        "uv tool run --from ruff rnu check .",
        "uvx",
        "uvx rnu check .",
        "uvx --from ruff rnu check .",
        "uv -- sycn",
        "uv pip -- isntall requests",
        "uv --help sycn",
        "uv --version sycn",
        "uv help pip isntall",
    ] {
        assert_eq!(
            suggest(input, &Context::default()),
            Outcome::NoMatch,
            "{input}"
        );
    }
}
