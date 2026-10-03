# the-heck

Fix a mistyped Git, GitHub CLI, Cargo, or uv command by typing **`heck`**:

```text
$ git stats --short
git: 'stats' is not a git command.

$ heck
  git status --short
```

Use Up/Down to choose a correction. **Enter opens it for editing; Enter again
runs it.** Bash uses a prefilled `heck>` prompt; Zsh uses your normal prompt.
Escape or Ctrl-C cancels the picker, and Ctrl-C cancels editing.

## Install and activate

From this checkout:

```sh
cargo install --path . --locked
```

Ensure `~/.cargo/bin` is on your `PATH`, then run the line for your current shell:

```sh
source <(command heck init bash)  # Bash
source <(command heck init zsh)   # Zsh
```

Add the appropriate line to `~/.bashrc` or `~/.zshrc` for future sessions.
After updating the binary, source that line again to refresh your current shell.

Bare `heck` repairs the last command in live shell history, skipping repeated
`heck` entries. History settings can omit commands or import them from other
sessions, so check the command shown in the picker.

Existing aliases and functions are preserved. If another alias or
function named `heck` exists, remove or rename it before activating the integration.
`heck --help` and other CLI arguments still go to the executable.

### Remove the integration

`heck_uninstall` removes this session's integration. Remove the initialization
line from your shell configuration to disable it in future sessions.

## Scope

- Corrects Git, GitHub CLI (`gh`), Cargo, and uv program names and commands.
- Supports nested Git commands under `remote`, `stash`, `worktree`, `submodule`,
  `bisect`, and `sparse-checkout`.
- Supports gh command groups, including `pr`, `issue`, `repo`, `run`, `workflow`,
  `release`, and deeper groups such as `repo autolink`.
- Supports GitHub's `gh stack` extension when installed.
- Preserves arguments, quotes, escapes, Unicode, and spacing.
- Supports simple, single-line commands with literal program/subcommand words.
- Runs in interactive Bash and Zsh on Unix, including WSL.

Examples of corrections:

| Mistyped command | Suggested command |
| --- | --- |
| `git remote ad origin URL` | `git remote add origin URL` |
| `git stash psuh -m "wip"` | `git stash push -m "wip"` |
| `git -C "my repo" stats` | `git -C "my repo" status` |
| `gh pr --repo owner/repo veiw 42` | `gh pr --repo owner/repo view 42` |
| `gh stack subimt` | `gh stack submit` |
| `gh stack rebsae --continue` | `gh stack rebase --continue` |
| `uv pip isntall requests` | `uv pip install requests` |
| `uv --project "my app" tool isntall ruff` | `uv --project "my app" tool install ruff` |

Git options before the command include `-C`, `-c`, `--git-dir`, and
`--work-tree`; gh supports `-R`/`--repo` before supported subcommands.
Heck preserves option spelling and values. Unknown options before a subcommand
are declined because their values could otherwise be mistaken for commands.
Git repository options must use fixed values, such as `-C "my repo"`;
shell expansions such as `-C "$REPO"` are declined.

Configured Git/gh aliases and installed gh extension names are preserved.
Heck reads their names locally, without running alias bodies or extensions.
Git alias lookup respects the selected repository and inline `-c` options.
If metadata lookup fails, corrections for that tool are skipped.
Cargo aliases and external `git-*` commands are not discovered.

Pipelines, redirections, compound commands, command substitutions, unfinished
quotes, and commands longer than 64 KiB are declined. Environment assignments
and Cargo toolchain selectors before the command are not supported.
Branch names, paths, flag typos, and errors requiring repository state are
outside the current scope.

Heck uses a local command catalog. It does not validate whether the selected
command will succeed, and it never runs a suggestion without your submission.

### uv

Supports the documented top-level commands and `pip`, `tool`, `python`, `cache`, `auth`,
and `self` groups from [uv 0.9.25](https://github.com/astral-sh/uv/tree/0.9.25),
including built-in aliases.

Global options such as `--project`, `--directory`, `--offline`, and
`-q`/`-v`/`-n` can appear before commands or between a group and its subcommand.
Option values, package names, and paths are preserved. Commands passed to
`uv run` and `uv tool run`, and all `uvx` arguments, are left untouched.
Suggestions use the local catalog without invoking uv.

### GitHub stacks

Install [GitHub's stack extension](https://github.com/github/gh-stack) separately
if you want to use it. It requires gh 2.0+ and Git 2.36+:

```sh
gh extension install github/gh-stack
```

Heck recognizes its setup, navigation, submission, synchronization, rebase,
merge, and management commands, including the `delete` alias for `unstack`.
It identifies the extension through `gh extension list`; another extension
named `stack` is kept opaque. The optional `gs` wrapper is not recognized.

The gh catalog follows version 2.86.0; the stack catalog follows
[revision d4ab7ab](https://github.com/github/gh-stack/tree/d4ab7ab47e5b3e3708a27c8c42abcdf4bc321419).
Older installations may support fewer commands. Suggestions do not contact
GitHub or change stack state.

## Advanced CLI

```sh
heck suggest -- 'git stats --short'
printf '%s' 'cargo bulid --release' | heck suggest --stdin
printf '%s' 'git stats --short' | heck pick --stdin
```

`suggest` prints complete candidates, one per line. `pick` shows a terminal menu
and prints only the accepted command, without a trailing newline. Both accept
one quoted command after `--` or exact command text through `--stdin`.
Use `printf '%s'` to avoid adding a newline.

The shell integration supplies facts about aliases, functions, and executable
names. Both shell and standalone invocations read local Git/gh aliases and gh
extensions; standalone invocations cannot inspect aliases or functions in your shell.

| Exit code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | No correction found |
| 2 | Invalid arguments or unsupported syntax |
| 3 | I/O failure or unavailable terminal |
| 130 | Picker cancelled |

## Development

Install Bash and Zsh, then run:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
bash -n shell/heck.bash
zsh -n shell/heck.zsh
```

On Unix, `cargo test` includes real terminal tests with temporary homes and
recording stand-ins for Git, gh, Cargo, and uv. Metadata tests also use temporary Git
repositories and a gh stand-in; they need no GitHub account or network access.
To try the CLI without installing it,
run `cargo build --locked` and prepend `$PWD/target/debug` to your `PATH`.
