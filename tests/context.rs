//! Local metadata lookup through the real CLI, without network access.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("bin")).unwrap();
        Self { root }
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(self.root.path())
            .env_clear()
            .env("HOME", self.root.path())
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.root.path().join("bin").display()),
            )
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    }

    fn git(&self, args: &[&str]) {
        let output = self.command("git").args(args).output().unwrap();
        assert!(output.status.success(), "{output:?}");
    }

    fn stub(&self, program: &str, body: &str) {
        let path = self.root.path().join("bin").join(program);
        fs::write(&path, body).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn suggest(&self, input: &str) -> Output {
        self.command(env!("CARGO_BIN_EXE_heck"))
            .args(["suggest", "--", input])
            .output()
            .unwrap()
    }
}

#[test]
fn git_alias_lookup_uses_the_selected_repository_and_inline_config() {
    let fixture = Fixture::new();
    fixture.git(&["init", "-q", "my repo"]);
    fixture.git(&["-C", "my repo", "config", "alias.stats", "!touch marker"]);
    fixture.git(&["config", "--global", "alias.restroe", "!touch marker"]);
    for input in [
        "git -C 'my repo' stats",
        "git -C my\\ repo stats",
        "git -Cmy\\ repo stats",
        "git --git-dir='my repo/.git' --work-tree \"my repo\" stats",
        "git -C 'my repo' -C . stats",
        "git -c alias.stats=status stats",
        "git -calias.stats=status stats",
        "git -c 'alias.stats=!touch marker' stats",
        "git restroe .",
    ] {
        let output = fixture.suggest(input);
        assert_eq!(output.status.code(), Some(1), "{input}: {output:?}");
        assert!(output.stdout.is_empty());
    }
    let output = fixture.suggest("gti -C 'my repo' stats");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"git -C 'my repo' stats\n");
    assert!(!fixture.root.path().join("marker").exists());
    assert!(!fixture.root.path().join("my repo/marker").exists());
}

#[test]
fn gh_aliases_and_extensions_are_opaque_except_the_official_stack() {
    let fixture = Fixture::new();
    fixture.stub("gh", r#"#!/bin/sh
case "$*" in
    "alias list") printf 'co: pr checkout\nrp: "!touch marker"\n"true": pr view\nmultiline: |\n    touch marker\n' ;;
    "extension list") printf 'gh stack\tgithub/gh-stack\tv1\ngh stacks\towner/gh-stacks\tv1\n' ;;
    *) touch "$HOME/marker"; exit 99 ;;
esac
"#);
    for input in [
        "gh rp ceate",
        "gh co ceate",
        "gh true",
        "gh multiline",
        "gh stacks subimt",
    ] {
        let output = fixture.suggest(input);
        assert_eq!(output.status.code(), Some(1), "{input}: {output:?}");
    }
    for (input, expected) in [
        ("gh stack subimt --open", "gh stack submit --open\n"),
        ("gh stakc view", "gh stack view\n"),
        ("gh pr ceate", "gh pr create\n"),
    ] {
        let output = fixture.suggest(input);
        assert!(output.status.success(), "{input}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
    assert!(!fixture.root.path().join("marker").exists());
}

#[test]
fn missing_or_unrelated_stack_extensions_are_not_suggested() {
    let fixture = Fixture::new();
    for extension_output in ["", "gh stack\tother/gh-stack\tv1\n", "gh stack\t\t\n"] {
        fixture.stub(
            "gh",
            &format!(
                r#"#!/bin/sh
case "$*" in
    "alias list") exit 0 ;;
    "extension list") printf '%s' '{extension_output}' ;;
    *) exit 99 ;;
esac
"#
            ),
        );

        for input in ["gh stack subimt", "gh stakc view"] {
            let output = fixture.suggest(input);
            assert_eq!(output.status.code(), Some(1), "{input}: {output:?}");
            assert!(output.stdout.is_empty());
        }
        let output = fixture.suggest("gh pr ceate");
        assert!(output.status.success(), "{output:?}");
    }
}

#[test]
fn failed_metadata_and_dynamic_git_context_do_not_produce_corrections() {
    let fixture = Fixture::new();
    fixture.stub("git", "#!/bin/sh\nexit 128\n");
    fixture.stub("gh", "#!/bin/sh\nexit 1\n");
    for input in ["git stats", "gh pr ceate"] {
        let output = fixture.suggest(input);
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stdout.is_empty());
    }
    fixture.stub("git", "#!/bin/sh\ntouch \"$HOME/marker\"\nexit 128\n");
    for input in ["git -C \"$REPO\" stats", "git -C $(touch marker) stats"] {
        let output = fixture.suggest(input);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(!fixture.root.path().join("marker").exists());
    }
}

#[test]
fn unavailable_gh_does_not_block_git_program_typos() {
    let fixture = Fixture::new();
    fixture.stub("gh", "#!/bin/sh\nexit 1\n");
    let output = fixture.suggest("gt stats");
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("git status\n"), "{text}");
    assert!(!text.lines().any(|line| line.starts_with("gh ")));
}
