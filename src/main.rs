use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;
use the_heck::{Context, Outcome};

const HELP: &str = "\
heck — suggest small repairs to Git and Cargo commands

Usage:
  heck suggest -- 'command text'
  heck suggest --stdin
  heck --help
  heck --version

suggest prints complete candidates, one per line, without executing them.

Exit codes: 0 success; 1 no match; 2 invalid/unsupported input; 3 I/O error.
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
        "suggest" => {}
        _ => return Err((2, "Unknown command. Run 'heck --help'.".into())),
    }

    let context = Context::default();
    let mut from_stdin = false;
    let mut input = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stdin" => from_stdin = true,
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
            let mut output = io::stdout().lock();
            for suggestion in suggestions {
                writeln!(output, "{}", suggestion.command).map_err(io_error)?;
            }
            Ok(0)
        }
    }
}

fn io_error(error: io::Error) -> (u8, String) {
    (3, error.to_string())
}
