mod picker;

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;
use the_heck::{Context, Outcome};

const HELP: &str = "\
heck — suggest small repairs to Bash and Zsh commands

Daily use (after shell setup):
  heck                         repair the previous command, then edit it

One-time setup for the current shell:
  source <(heck init bash)      Bash
  source <(heck init zsh)       Zsh

Other commands:
  heck suggest -- 'command text'
  heck suggest --stdin
  heck pick    --stdin
  heck init bash|zsh
  heck --help
  heck --version

suggest prints complete candidates, one per line.
pick offers a terminal menu and prints only the accepted command.
Accepting opens an editable prompt; press Enter again to run the command.
init enables bare heck in your shell.

Exit codes: 0 success; 1 no match; 2 invalid/unsupported input;
            3 I/O or terminal error; 130 cancelled.
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err((code, message)) => {
            eprintln!("heck: {message}");
            ExitCode::from(code)
        }
    }
}

type CliResult = Result<u8, (u8, String)>;

fn run() -> CliResult {
    let mut args = std::env::args().skip(1);
    let Some(action) = args.next() else {
        print!("{HELP}");
        return Ok(0);
    };
    match action.as_str() {
        "--help" | "-h" => {
            print!("{HELP}");
            return Ok(0);
        }
        "--version" | "-V" => {
            println!("heck {}", env!("CARGO_PKG_VERSION"));
            return Ok(0);
        }
        "init" => {
            let script = match args.next().as_deref() {
                Some("bash") => include_str!("../shell/heck.bash"),
                Some("zsh") => include_str!("../shell/heck.zsh"),
                _ => return Err((2, "Use 'heck init bash' or 'heck init zsh'.".into())),
            };
            if args.next().is_some() {
                return Err((2, "Unexpected argument to init.".into()));
            }
            io::stdout()
                .write_all(script.as_bytes())
                .map_err(io_error)?;
            return Ok(0);
        }
        "suggest" | "pick" => {}
        _ => return Err((2, "Unknown command. Run 'heck --help'.".into())),
    }

    let mut context = Context::default();
    let mut from_stdin = false;
    let mut input = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stdin" => from_stdin = true,
            // Internal facts supplied by shell integration, not correction instructions.
            "--program-known" => context.program_known = true,
            "--program-shadowed" => context.program_shadowed = true,
            "--" => {
                input = args.next();
                if input.is_none() || args.next().is_some() {
                    return Err((2, "Pass exactly one quoted command after '--'.".into()));
                }
                break;
            }
            "--help" | "-h" => {
                print!("{HELP}");
                return Ok(0);
            }
            _ => return Err((2, "Unknown option. Pass command text after '--'.".into())),
        }
    }
    if from_stdin == input.is_some() {
        return Err((2, "Use either --stdin or -- 'command text'.".into()));
    }
    let input = if from_stdin {
        if io::stdin().is_terminal() {
            return Err((
                2,
                "Pipe command text to --stdin, or pass it after '--'.".into(),
            ));
        }
        let mut text = String::new();
        io::stdin()
            .take(65_537)
            .read_to_string(&mut text)
            .map_err(io_error)?;
        text
    } else {
        input.unwrap_or_default()
    };
    match the_heck::suggest(&input, &context) {
        Outcome::NoMatch => {
            eprintln!("heck: No correction found.");
            Ok(1)
        }
        Outcome::Unsupported(message) => Err((2, message.into())),
        Outcome::Suggestions(suggestions) => {
            if action == "suggest" {
                let mut output = io::stdout().lock();
                for suggestion in suggestions {
                    writeln!(output, "{}", suggestion.command).map_err(io_error)?;
                }
                Ok(0)
            } else {
                match picker::pick(&input, &suggestions).map_err(io_error)? {
                    Some(index) => {
                        io::stdout()
                            .write_all(suggestions[index].command.as_bytes())
                            .map_err(io_error)?;
                        Ok(0)
                    }
                    None => Ok(130),
                }
            }
        }
    }
}

fn io_error(error: io::Error) -> (u8, String) {
    (3, error.to_string())
}
