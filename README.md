# the-heck

Fix a mistyped Git or Cargo command by typing **`heck`**:

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

- Corrects Git and Cargo program names and top-level subcommands.
- Preserves arguments, quotes, escapes, Unicode, and spacing.
- Supports simple, single-line commands with literal program/subcommand words.
- Runs in interactive Bash and Zsh on Unix, including WSL.

Pipelines, redirections, compound commands, command substitutions, unfinished
quotes, and commands longer than 64 KiB are declined. Environment assignments,
global options, and toolchain selectors before the subcommand are not supported.
Project-specific Git/Cargo aliases and extension commands are not discovered.

Heck uses a local command catalog. It does not validate whether the selected
command will succeed, and it never runs a suggestion without your submission.

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
names; standalone invocations only know the supplied text and built-in catalog.

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
recording stand-ins for Git and Cargo. To try the CLI without installing it,
run `cargo build --locked` and prepend `$PWD/target/debug` to your `PATH`.
