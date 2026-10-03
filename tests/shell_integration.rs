//! End-to-end Bash/Zsh behavior, driven through real terminals by cargo test.
#![cfg(unix)]

mod support;

use std::os::unix::process::CommandExt;
use std::process::Command;

use support::{executable, run, Session, BINARY, SHELLS};

#[test]
fn second_candidate_is_returned() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        session.command("git stat --short");
        let baseline = session.runs();
        session.send("heck\r");
        session.picker();
        session.send("\x1b[B");
        session.select_for_editing();
        assert_eq!(session.capture(), "git stash --short");
        assert_eq!(session.runs(), baseline);
    }
}

#[test]
fn aliases_and_functions_are_not_rewritten() {
    for shell in SHELLS {
        for (definition, text) in [
            ("alias gti=git", "gti stats"),
            ("git() { :; }", "git stats"),
        ] {
            let mut session = Session::configured(shell, "emacs", definition);
            session.command(text);
            let baseline = session.runs();
            assert!(session.command("heck").contains("No correction found"));
            assert_eq!(session.capture(), "");
            assert_eq!(session.runs(), baseline);
        }
    }
}

#[test]
fn existing_executable_is_not_rewritten() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        executable(&session.root.path().join("bin/gti"), "#!/bin/sh\nexit 0\n");
        session.command("gti stats");
        assert!(session.command("heck").contains("No correction found"));
        assert_eq!(session.capture(), "");
    }
}

#[test]
fn picker_without_a_controlling_terminal() {
    let mut command = Command::new(BINARY);
    command.args(["pick", "--", "git stats"]);
    // SAFETY: setsid is async-signal-safe and detaches this child from the
    // parent's controlling terminal, including when cargo test runs in a TTY.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("interactive terminal"));
}

#[test]
fn bare_heck_select_edit_then_submit() {
    for shell in SHELLS {
        for mode in ["emacs", "vi"] {
            let mut session = Session::configured(shell, mode, "");
            session.command("  git stats --short \"日本語 file\"  ");
            let baseline = session.runs();
            session.send("heck\r");
            session.picker();
            session.select_for_editing();
            let expected = "  git status --short \"日本語 file\"  ";
            assert_eq!(session.capture(), expected);
            assert_eq!(session.runs(), baseline);
            session.send("--branch");
            assert_eq!(session.capture(), format!("{expected}--branch"));
            session.send("\r");
            session.prompt();
            let runs = session.runs();
            assert_eq!(runs.len(), 2);
            assert_eq!(runs[1], ["status", "--short", "日本語 file", "--branch"]);
            session.command("_heck_previous_command > \"$HOME/previous\"");
            // Shell history can normalize leading spaces after submission.
            assert_eq!(
                session.read_file("previous").trim_start(),
                format!("{expected}--branch").trim_start()
            );
        }
    }
}

#[test]
fn bare_heck_can_cancel_selection_and_retry() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        session.command("cargo bulid --release");
        let baseline = session.runs();
        for cancel in ["\x1b", "\x03"] {
            session.send("heck\r");
            session.picker();
            session.send(cancel);
            session.prompt();
            assert_eq!(session.capture(), "");
            assert_eq!(session.runs(), baseline);
        }
        session.send("heck\r");
        session.picker();
        session.select_for_editing();
        assert_eq!(session.capture(), "cargo build --release");
    }
}

#[test]
fn bare_heck_can_cancel_editing() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        session.command("cargo bulid");
        let baseline = session.runs();
        session.send("heck\r");
        session.picker();
        session.select_for_editing();
        session.send("\x03");
        session.prompt();
        assert_eq!(session.capture(), "");
        assert_eq!(session.runs(), baseline);
    }
}

#[test]
fn bare_heck_empty_history_and_cli_forwarding() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        let output = session.command("heck");
        assert!(output.contains("No command in recent shell history"));
        assert_eq!(session.capture(), "");
        let output = session.command("heck --version");
        assert!(output.contains(concat!("heck ", env!("CARGO_PKG_VERSION"))));
    }
}

#[test]
fn existing_heck_alias_or_function_is_preserved() {
    for shell in SHELLS {
        for definition in [
            "function heck { printf 'KEPT_FUNCTION\\n'; }",
            "alias heck=\"printf 'KEPT_ALIAS\\n'\"",
        ] {
            let mut session = Session::configured(shell, "emacs", definition);
            assert!(session.command("heck").contains("KEPT_"));
        }
    }
}

#[test]
fn source_upgrades_a_legacy_session_without_bare_heck() {
    for shell in SHELLS {
        let legacy = shell.choose(
            "_HECK_BASH_INITIALIZED=1\n_heck_pick() { printf STALE_HELPER; }",
            "_HECK_ZSH_INITIALIZED=1\n_heck_pick() { printf STALE_HELPER; }",
        );
        let mut session = Session::configured(shell, "emacs", legacy);
        session.command("git stat");
        session.send("heck\r");
        session.picker();
        session.select_for_editing();
        let buffer = session.capture();
        assert!(buffer.starts_with("git "));
        assert_ne!(buffer, "git stat");
        assert_eq!(session.runs().len(), 1);
    }
}

#[test]
fn reinitialization_and_uninstall() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        for _ in 0..2 {
            let output = session.command(&format!("source <(heck init {})", shell.name()));
            assert!(!output.contains("Keeping the existing"));
        }
        session.command("cargo bulid");
        session.send("heck\r");
        session.picker();
        session.select_for_editing();
        assert_eq!(session.capture(), "cargo build");
        session.send("\x03");
        session.prompt();
        session.command("heck_uninstall");
        let output = session.command(shell.choose("type heck", "whence -w heck"));
        assert!(!output.contains("function"));
    }
}

#[test]
fn refresh_preserves_user_replacements() {
    for shell in SHELLS {
        let mut session = Session::new(shell);
        session.command("function heck { printf 'KEPT_FUNCTION\\n'; }");
        session.command(&format!("source <(command heck init {})", shell.name()));
        assert!(session.command("heck").contains("KEPT_FUNCTION\r\n"));
        session.command("heck_uninstall");
        assert!(session.command("heck").contains("KEPT_FUNCTION\r\n"));
    }
}

#[test]
fn nested_tool_corrections_wait_for_submission() {
    for shell in SHELLS {
        for (input, expected, args) in [
            (
                "gh stack rebsae --continue",
                "gh stack rebase --continue",
                vec!["stack", "rebase", "--continue"],
            ),
            (
                "uv --offline tool isntall 'ruff==0.14.0'",
                "uv --offline tool install 'ruff==0.14.0'",
                vec!["--offline", "tool", "install", "ruff==0.14.0"],
            ),
        ] {
            let mut session = Session::new(shell);
            session.command(input);
            let baseline = session.runs();
            session.send("heck\r");
            session.picker();
            session.select_for_editing();
            assert_eq!(session.capture(), expected);
            assert_eq!(session.runs(), baseline);
            session.send("\r");
            session.prompt();
            let runs = session.runs();
            assert_eq!(runs.len(), baseline.len() + 1);
            assert_eq!(runs.last().unwrap(), &args);
        }
    }
}
