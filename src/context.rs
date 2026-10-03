//! Local metadata collection, kept separate from deterministic suggestions.
use std::process::{Command, Output};

use crate::{lexer, programs, syntax, Context};

/// Populate only the metadata relevant to this command. Never execute the
/// input, alias bodies, or extensions. Invalid syntax is left to `suggest`.
pub fn discover(input: &str, context: &mut Context) -> Result<(), &'static str> {
    let Ok(tokens) = lexer::tokenize(input) else {
        return Ok(());
    };
    let Ok(programs) = programs(input, &tokens, context) else {
        return Ok(());
    };
    let mut failure = None;
    for (program, _) in programs {
        let Ok(position) = syntax::command(input, &tokens, 1, program) else {
            continue;
        };
        if position.index.is_none() {
            continue;
        }
        let result = match program {
            "git" => git_aliases(&position.git_args).map(|aliases| context.git_aliases = aliases),
            "gh" => gh_metadata(context),
            _ => Ok(()),
        };
        if let Err(message) = result {
            context.unavailable_programs.push(program.to_owned());
            failure = Some(message);
        }
    }
    failure.map_or(Ok(()), Err)
}

fn git_aliases(args: &[String]) -> Result<Vec<String>, &'static str> {
    let output = Command::new("git")
        .args(args)
        .args([
            "config",
            "--null",
            "--name-only",
            "--get-regexp",
            "^alias[.]",
        ])
        .output()
        .map_err(|_| "Could not read Git aliases.")?;
    if !(output.status.success() || output.status.code() == Some(1) && output.stdout.is_empty()) {
        return Err("Could not read Git aliases.");
    }
    let text = String::from_utf8(output.stdout).map_err(|_| "Could not read Git aliases.")?;
    text.split('\0')
        .filter(|name| !name.is_empty())
        .map(|name| name.strip_prefix("alias.").map(str::to_owned))
        .collect::<Option<Vec<_>>>()
        .ok_or("Could not read Git aliases.")
}

fn gh_metadata(context: &mut Context) -> Result<(), &'static str> {
    let output = gh()
        .args(["alias", "list"])
        .output()
        .map_err(|_| "Could not read gh aliases.")?;
    context.gh_aliases = listing(output, "no aliases configured")
        .and_then(|text| alias_names(&text))
        .ok_or("Could not read gh aliases.")?;

    let output = gh()
        .args(["extension", "list"])
        .output()
        .map_err(|_| "Could not list gh extensions.")?;
    let (names, stack) = listing(output, "no installed extensions found")
        .and_then(|text| extension_names(&text))
        .ok_or("Could not list gh extensions.")?;
    context.gh_extensions = names;
    context.gh_stack = stack;
    Ok(())
}

fn gh() -> Command {
    let mut command = Command::new("gh");
    command
        .env_remove("GH_FORCE_TTY")
        .env("NO_COLOR", "1")
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_NO_UPDATE_NOTIFIER", "1")
        .env("GH_NO_EXTENSION_UPDATE_NOTIFIER", "1");
    command
}

fn listing(output: Output, empty_message: &str) -> Option<String> {
    if output.status.success()
        || (output.status.code() == Some(1)
            && output.stdout.is_empty()
            && String::from_utf8_lossy(&output.stderr).trim() == empty_message)
    {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

// gh emits a YAML mapping. We only need top-level keys which could be literal
// command words. Indented continuation lines belong to alias bodies.
fn alias_names(text: &str) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with(char::is_whitespace) || line == "{}" {
            continue;
        }
        let (name, _) = line.split_once(':')?;
        names.push(name.trim().trim_matches(['\'', '"']).to_owned());
    }
    Some(names)
}

// With stdout piped, gh emits tab-separated name, repository, and version.
fn extension_names(text: &str) -> Option<(Vec<String>, bool)> {
    let mut names = Vec::new();
    let mut stack = false;
    for line in text.lines().filter(|line| !line.is_empty()) {
        let mut fields = line.split('\t');
        let name = fields.next()?.strip_prefix("gh ")?;
        let repo = fields.next()?;
        stack |= name == "stack" && repo.eq_ignore_ascii_case("github/gh-stack");
        names.push(name.to_owned());
    }
    Some((names, stack))
}
