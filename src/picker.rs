use console::{measure_text_width, Key, Term};
use std::io;
use the_heck::Suggestion;

#[cfg(unix)]
fn terminal() -> io::Result<Term> {
    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "The picker needs an interactive terminal. Use 'heck suggest' for scripts.",
            )
        })?;
    Ok(Term::read_write_pair(tty.try_clone()?, tty))
}

#[cfg(not(unix))]
fn terminal() -> io::Result<Term> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "The picker currently supports Bash and Zsh on Unix (including WSL).",
    ))
}

struct CursorGuard<'a>(&'a Term);
impl Drop for CursorGuard<'_> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
        let _ = self.0.flush();
    }
}

pub fn pick(original: &str, suggestions: &[Suggestion]) -> io::Result<Option<usize>> {
    let term = terminal()?;
    let _cursor = CursorGuard(&term);
    term.hide_cursor()?;
    let mut selected = 0;
    loop {
        let suggestion = &suggestions[selected];
        let mut highlighted = String::new();
        let mut end = 0;
        for edit in &suggestion.edits {
            highlighted.push_str(&original[end..edit.range.start]);
            highlighted.push_str(
                &term
                    .style()
                    .bold()
                    .underlined()
                    .apply_to(&edit.replacement)
                    .to_string(),
            );
            end = edit.range.end;
        }
        highlighted.push_str(&original[end..]);
        let lines = [
            format!("Repair: {}", original.replace('\t', "    ")),
            format!("Suggestion {} of {}:", selected + 1, suggestions.len()),
            format!("  {}", highlighted.replace('\t', "    ")),
            format!("  {}", suggestion.reason),
            "Up/Down: choose | Enter: accept for editing | Esc/Ctrl-C: cancel".into(),
        ];
        let width = usize::from(term.size().1).max(1);
        let mut rows = 0;
        for line in &lines {
            term.write_line(line)?;
            rows += measure_text_width(line).max(1).div_ceil(width);
        }
        term.flush()?;
        let key = term.read_key();
        term.clear_last_lines(rows)?;
        match key? {
            Key::Enter => return Ok(Some(selected)),
            Key::Escape | Key::Char('\u{3}') | Key::Char('q') => return Ok(None),
            Key::ArrowDown | Key::Tab | Key::Char('j') => {
                selected = (selected + 1) % suggestions.len()
            }
            Key::ArrowUp | Key::BackTab | Key::Char('k') => {
                selected = (selected + suggestions.len() - 1) % suggestions.len();
            }
            _ => {}
        }
    }
}
