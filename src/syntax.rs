use crate::lexer::{self, Token};

pub(crate) struct Position {
    pub index: Option<usize>,
    pub git_args: Vec<String>,
}

/// Skip only options whose arity is known. Once a leaf command is reached its
/// remaining tokens are opaque, including options and their values.
pub(crate) fn command(
    input: &str,
    tokens: &[Token],
    mut index: usize,
    path: &str,
) -> Result<Position, &'static str> {
    let uv = path == "uv" || path.starts_with("uv ");
    let mut git_args = Vec::new();
    let index = loop {
        let Some(token) = tokens.get(index) else {
            break None;
        };
        let raw = &input[token.range.clone()];
        if matches!(raw, "--" | "-h" | "--help")
            || (raw == "--version" && !path.starts_with("uv "))
            || (path == "uv" && raw == "-V")
        {
            break None;
        }
        if !raw.starts_with('-') {
            if path == "cargo" && raw.starts_with('+') {
                return Err("Cargo toolchain selectors are not supported yet.");
            }
            break Some(index);
        }
        // A flag here selects stash's implicit push command. Everything after
        // it can be a message or pathspec, even if it resembles a command.
        if path == "git stash" {
            break None;
        }
        let (name, inline) =
            if path == "git" && raw.len() > 2 && (raw.starts_with("-C") || raw.starts_with("-c")) {
                (&raw[..2], Some(&raw[2..]))
            } else if path.starts_with("gh") && raw.starts_with("-R") && raw.len() > 2 {
                ("-R", Some(&raw[2..]))
            } else if let Some((name, value)) = raw.split_once('=') {
                (name, Some(value))
            } else {
                (raw, None)
            };
        let git_value = path == "git"
            && matches!(
                name,
                "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace"
            );
        let gh_repo = matches!(
            path,
            "gh" | "gh pr"
                | "gh issue"
                | "gh release"
                | "gh repo"
                | "gh repo autolink"
                | "gh repo deploy-key"
                | "gh run"
                | "gh workflow"
                | "gh cache"
                | "gh label"
                | "gh secret"
                | "gh variable"
                | "gh ruleset"
        ) && matches!(name, "-R" | "--repo");
        let uv_value = uv
            && matches!(
                name,
                "--cache-dir"
                    | "--color"
                    | "--allow-insecure-host"
                    | "--trusted-host"
                    | "--directory"
                    | "--project"
                    | "--config-file"
            );
        if git_value || gh_repo || uv_value {
            let value = match inline {
                Some(value) => value,
                None => {
                    index += 1;
                    let value = tokens
                        .get(index)
                        .ok_or("Missing value for an option before the subcommand.")?;
                    &input[value.range.clone()]
                }
            };
            if git_value {
                let value = lexer::value(value).ok_or(
                    "Git options before the subcommand must use values without shell expansions.",
                )?;
                git_args.push(name.to_owned());
                git_args.push(value);
            }
        } else {
            let flag = match path {
                "git" => matches!(
                    name,
                    "-p" | "--paginate"
                        | "-P"
                        | "--no-pager"
                        | "--bare"
                        | "--no-replace-objects"
                        | "--literal-pathspecs"
                        | "--glob-pathspecs"
                        | "--noglob-pathspecs"
                        | "--icase-pathspecs"
                        | "--no-optional-locks"
                ),
                "git remote" => matches!(name, "-v" | "--verbose" | "--no-verbose"),
                "git submodule" => matches!(name, "-q" | "--quiet" | "--cached"),
                _ if uv => {
                    matches!(
                        name,
                        "--no-cache"
                            | "--quiet"
                            | "--verbose"
                            | "--managed-python"
                            | "--no-managed-python"
                            | "--no-python-downloads"
                            | "--native-tls"
                            | "--offline"
                            | "--no-progress"
                            | "--no-config"
                            | "--preview"
                            | "--no-preview"
                    ) || name.strip_prefix('-').is_some_and(|shorts| {
                        !shorts.is_empty()
                            && shorts.chars().all(|flag| matches!(flag, 'n' | 'q' | 'v'))
                    })
                }
                _ => false,
            };
            if !flag || inline.is_some() {
                return Err("Unknown option before the subcommand.");
            }
            if path == "git" && name == "--bare" {
                git_args.push(name.to_owned());
            }
        }
        index += 1;
    };
    Ok(Position { index, git_args })
}
