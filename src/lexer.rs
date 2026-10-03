use std::ops::Range;

#[derive(Debug)]
pub(crate) struct Token {
    pub range: Range<usize>,
    pub literal: bool,
}

/// Recognize a deliberately small shared Bash/Zsh grammar. Argument text is
/// opaque; we only need its boundaries to avoid editing inside quoted strings.
pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, &'static str> {
    if input.len() > 65_536 {
        return Err("Commands longer than 64 KiB are not supported.");
    }
    if input.chars().any(|c| c.is_control() && c != '\t') {
        return Err("Only single-line commands without control characters are supported.");
    }
    let mut tokens = Vec::new();
    let mut start = None;
    let mut quote = None;
    let mut escaped = false;
    let mut literal = true;
    let mut chars = input.char_indices().peekable();
    while let Some((offset, ch)) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if quote == Some('\'') {
            if ch == '\'' {
                quote = None;
            }
            continue;
        }
        if ch == '\\' {
            start.get_or_insert(offset);
            escaped = true;
            literal = false;
            continue;
        }
        if ch == '`' || (ch == '$' && chars.peek().is_some_and(|(_, next)| *next == '(')) {
            return Err("Command substitutions are not supported.");
        }
        if quote == Some('"') {
            if ch == '"' {
                quote = None;
            }
            continue;
        }
        if ch == ' ' || ch == '\t' {
            if let Some(begin) = start.take() {
                tokens.push(Token {
                    range: begin..offset,
                    literal,
                });
                literal = true;
            }
            continue;
        }
        if matches!(ch, ';' | '|' | '&' | '<' | '>' | '(' | ')') {
            return Err("Pipelines, redirections, and compound commands are not supported yet.");
        }
        if ch == '#' && start.is_none() {
            return Err("Shell comments are not supported yet.");
        }
        start.get_or_insert(offset);
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            literal = false;
        } else if !ch.is_ascii_alphanumeric() && !matches!(ch, '-' | '_' | '=') {
            literal = false;
        }
    }
    if escaped || quote.is_some() {
        return Err("The command contains an unfinished quote or escape.");
    }
    if let Some(begin) = start {
        tokens.push(Token {
            range: begin..input.len(),
            literal,
        });
    }
    Ok(tokens)
}

/// Decode a fixed shell word for a read-only metadata query. This deliberately
/// rejects expansion; it never asks a shell to interpret user input.
pub(crate) fn value(raw: &str) -> Option<String> {
    let mut result = String::new();
    let mut quote = None;
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if quote == Some('\'') {
            if ch == '\'' {
                quote = None;
            } else {
                result.push(ch);
            }
        } else if ch == '\\' {
            let next = chars.next()?;
            if quote == Some('"') && !matches!(next, '$' | '`' | '"' | '\\') {
                result.push('\\');
            }
            result.push(next);
        } else if quote == Some('"') {
            match ch {
                '"' => quote = None,
                '$' | '`' => return None,
                _ => result.push(ch),
            }
        } else {
            match ch {
                '\'' | '"' => quote = Some(ch),
                '$' | '`' | '~' | '*' | '?' | '[' | ']' | '{' | '}' => return None,
                _ => result.push(ch),
            }
        }
    }
    quote.is_none().then_some(result)
}
