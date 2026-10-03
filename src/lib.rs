//! Suggest small edits to shell commands without evaluating or executing them.
//!
//! Only simple commands in the shared Bash/Zsh syntax subset are supported.
//! Edits use UTF-8 byte offsets into the original input.
mod lexer;

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range<usize>,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub command: String,
    pub edits: Vec<TextEdit>,
    pub reason: String,
    pub distance: usize,
}

/// Facts supplied by the active shell. No commands are run by the engine.
#[derive(Debug, Default)]
pub struct Context {
    /// The original program already resolves to an executable or shell builtin.
    pub program_known: bool,
    /// The original program is an alias or function; its grammar is unknown.
    pub program_shadowed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Suggestions(Vec<Suggestion>),
    NoMatch,
    Unsupported(&'static str),
}

// These are command words, never command lines containing flags or arguments.
const GIT: &[&str] = &[
    "add",
    "am",
    "annotate",
    "apply",
    "archive",
    "bisect",
    "blame",
    "branch",
    "bugreport",
    "bundle",
    "cat-file",
    "check-attr",
    "check-ignore",
    "check-mailmap",
    "check-ref-format",
    "checkout",
    "checkout-index",
    "cherry",
    "cherry-pick",
    "clean",
    "clone",
    "column",
    "commit",
    "commit-graph",
    "commit-tree",
    "config",
    "count-objects",
    "credential",
    "credential-cache",
    "credential-store",
    "daemon",
    "describe",
    "diagnose",
    "diff",
    "diff-files",
    "diff-index",
    "diff-tree",
    "difftool",
    "fast-export",
    "fast-import",
    "fetch",
    "fetch-pack",
    "filter-branch",
    "fmt-merge-msg",
    "for-each-ref",
    "for-each-repo",
    "format-patch",
    "fsck",
    "fsck-objects",
    "gc",
    "get-tar-commit-id",
    "grep",
    "hash-object",
    "help",
    "hook",
    "http-backend",
    "http-fetch",
    "http-push",
    "imap-send",
    "index-pack",
    "init",
    "init-db",
    "instaweb",
    "interpret-trailers",
    "log",
    "ls-files",
    "ls-remote",
    "ls-tree",
    "mailinfo",
    "mailsplit",
    "maintenance",
    "merge",
    "merge-base",
    "merge-file",
    "merge-index",
    "merge-tree",
    "mergetool",
    "mktag",
    "mktree",
    "multi-pack-index",
    "mv",
    "name-rev",
    "notes",
    "pack-objects",
    "pack-redundant",
    "pack-refs",
    "patch-id",
    "pickaxe",
    "prune",
    "prune-packed",
    "pull",
    "push",
    "quiltimport",
    "range-diff",
    "read-tree",
    "rebase",
    "receive-pack",
    "reflog",
    "remote",
    "repack",
    "replace",
    "request-pull",
    "rerere",
    "reset",
    "restore",
    "rev-list",
    "rev-parse",
    "revert",
    "rm",
    "send-pack",
    "shell",
    "shortlog",
    "show",
    "show-branch",
    "show-index",
    "show-ref",
    "sparse-checkout",
    "stage",
    "stash",
    "status",
    "stripspace",
    "submodule",
    "subtree",
    "switch",
    "symbolic-ref",
    "tag",
    "unpack-file",
    "unpack-objects",
    "update-index",
    "update-ref",
    "update-server-info",
    "upload-archive",
    "upload-pack",
    "var",
    "verify-commit",
    "verify-pack",
    "verify-tag",
    "version",
    "whatchanged",
    "worktree",
    "write-tree",
];
const CARGO: &[&str] = &[
    "add",
    "b",
    "bench",
    "build",
    "c",
    "check",
    "clean",
    "clippy",
    "config",
    "d",
    "doc",
    "fetch",
    "fix",
    "fmt",
    "generate-lockfile",
    "git-checkout",
    "help",
    "info",
    "init",
    "install",
    "locate-project",
    "login",
    "logout",
    "metadata",
    "miri",
    "new",
    "owner",
    "package",
    "pkgid",
    "publish",
    "r",
    "read-manifest",
    "remove",
    "report",
    "rm",
    "run",
    "rustc",
    "rustdoc",
    "search",
    "t",
    "test",
    "tree",
    "uninstall",
    "update",
    "vendor",
    "verify-project",
    "version",
    "yank",
];

fn commands(program: &str) -> Option<&'static [&'static str]> {
    match program {
        "git" => Some(GIT),
        "cargo" => Some(CARGO),
        _ => None,
    }
}

/// Return ranked, complete suggestions. Everything outside the edited words is
/// copied verbatim. Unknown syntax is declined rather than interpreted.
pub fn suggest(input: &str, context: &Context) -> Outcome {
    let tokens = match lexer::tokenize(input) {
        Ok(tokens) => tokens,
        Err(message) => return Outcome::Unsupported(message),
    };
    let Some(program_token) = tokens.first() else {
        return Outcome::NoMatch;
    };
    if context.program_shadowed {
        return Outcome::NoMatch;
    }
    if !program_token.literal {
        return Outcome::Unsupported("Use a literal, unquoted program name.");
    }
    let program = &input[program_token.range.clone()];
    if program.contains('=') {
        return Outcome::Unsupported(
            "Environment assignments before commands are not supported yet.",
        );
    }
    if context.program_known && commands(program).is_none() {
        return Outcome::NoMatch;
    }
    let programs = matches(program, &["cargo", "git"]);
    if programs.is_empty() {
        return Outcome::NoMatch;
    }
    if let Some(token) = tokens.get(1) {
        if !token.literal {
            return Outcome::Unsupported("Use a literal, unquoted subcommand.");
        }
        if input[token.range.clone()].starts_with(['-', '+']) {
            return Outcome::Unsupported(
                "Options or toolchain selectors before the subcommand are not supported yet.",
            );
        }
    }

    let mut results = Vec::new();
    for (fixed_program, program_distance) in programs {
        let subcommands = match tokens.get(1) {
            Some(token) => matches(
                &input[token.range.clone()],
                commands(fixed_program).unwrap(),
            ),
            None => vec![("", 0)],
        };
        for (fixed_subcommand, subcommand_distance) in subcommands {
            let mut edits = Vec::new();
            let mut reasons = Vec::new();
            if program_distance > 0 {
                edits.push(TextEdit {
                    range: program_token.range.clone(),
                    replacement: fixed_program.to_owned(),
                });
                reasons.push(format!("Program: {program} → {fixed_program}"));
            }
            if subcommand_distance > 0 {
                let token = &tokens[1];
                let subcommand = &input[token.range.clone()];
                edits.push(TextEdit {
                    range: token.range.clone(),
                    replacement: fixed_subcommand.to_owned(),
                });
                reasons.push(format!(
                    "{fixed_program} subcommand: {subcommand} → {fixed_subcommand}"
                ));
            }
            if !edits.is_empty() {
                results.push(candidate(
                    input,
                    edits,
                    reasons,
                    program_distance + subcommand_distance,
                ));
            }
        }
    }
    results.sort_by(|a, b| {
        (a.distance, a.edits.len(), &a.command).cmp(&(b.distance, b.edits.len(), &b.command))
    });
    results.truncate(8);
    if results.is_empty() {
        Outcome::NoMatch
    } else {
        Outcome::Suggestions(results)
    }
}

fn candidate(
    input: &str,
    edits: Vec<TextEdit>,
    reasons: Vec<String>,
    distance: usize,
) -> Suggestion {
    let mut command = input.to_owned();
    // The edits are ordered by their original position. Apply from the end so
    // earlier byte offsets remain valid when replacements have different sizes.
    for edit in edits.iter().rev() {
        command.replace_range(edit.range.clone(), &edit.replacement);
    }
    Suggestion {
        command,
        edits,
        reason: reasons.join("; "),
        distance,
    }
}

fn matches<'a>(word: &'a str, candidates: &[&'a str]) -> Vec<(&'a str, usize)> {
    if candidates.contains(&word) {
        return vec![(word, 0)];
    }
    if word.len() < 2 || !word.is_ascii() {
        return Vec::new();
    }
    let limit = if word.len() <= 3 { 1 } else { 2 };
    candidates
        .iter()
        .filter_map(|&candidate| {
            // One-letter Cargo aliases are valid input, but poor suggestions.
            if candidate.len() < 2 || word.len().abs_diff(candidate.len()) > limit {
                return None;
            }
            let distance = edit_distance(word.as_bytes(), candidate.as_bytes());
            (distance > 0 && distance <= limit).then_some((candidate, distance))
        })
        .collect()
}

// Optimal string alignment distance: adjacent transpositions count as one typo.
// The vocabulary is small, so a matrix is simpler than building a search index.
fn edit_distance(a: &[u8], b: &[u8]) -> usize {
    let mut rows = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, value) in rows[0].iter_mut().enumerate() {
        *value = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            rows[i][j] = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]));
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                rows[i][j] = rows[i][j].min(rows[i - 2][j - 2] + 1);
            }
        }
    }
    rows[a.len()][b.len()]
}
