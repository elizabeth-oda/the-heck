//! Suggest small edits to shell commands without evaluating or executing them.
//!
//! Only simple commands in the shared Bash/Zsh syntax subset are supported.
//! Edits use UTF-8 byte offsets into the original input.
mod catalog;
pub mod context;
mod lexer;
mod syntax;

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

/// Facts supplied by the active shell and local metadata lookup.
/// The suggestion engine itself performs no I/O.
#[derive(Debug, Default)]
pub struct Context {
    /// The original program resolves to an executable or shell builtin.
    pub program_known: bool,
    /// The original program is a shell alias or function with unknown grammar.
    pub program_shadowed: bool,
    pub git_aliases: Vec<String>,
    pub gh_aliases: Vec<String>,
    pub gh_extensions: Vec<String>,
    /// The installed "stack" extension is github/gh-stack.
    pub gh_stack: bool,
    /// Tools whose local metadata could not be read.
    pub unavailable_programs: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Suggestions(Vec<Suggestion>),
    NoMatch,
    Unsupported(&'static str),
}

/// Return ranked, complete suggestions. Everything outside the edited words is
/// copied verbatim. Unknown syntax is declined rather than interpreted.
pub fn suggest(input: &str, context: &Context) -> Outcome {
    let tokens = match lexer::tokenize(input) {
        Ok(tokens) => tokens,
        Err(message) => return Outcome::Unsupported(message),
    };
    let programs = match programs(input, &tokens, context) {
        Ok(programs) => programs,
        Err(message) => return Outcome::Unsupported(message),
    };

    let mut search = Search {
        input,
        tokens: &tokens,
        context,
        results: Vec::new(),
    };
    let mut unsupported = None;
    for (fixed, distance) in programs {
        if context
            .unavailable_programs
            .iter()
            .any(|name| name == fixed)
        {
            continue;
        }
        let mut route = Route {
            path: fixed.to_owned(),
            next: 1,
            edits: Vec::new(),
            reasons: Vec::new(),
            distance,
        };
        if distance > 0 {
            let program_token = &tokens[0];
            let program = &input[program_token.range.clone()];
            route.edits.push(TextEdit {
                range: program_token.range.clone(),
                replacement: fixed.to_owned(),
            });
            route.reasons.push(format!("Program: {program} → {fixed}"));
        }
        if let Err(message) = search.walk(route) {
            unsupported = Some(message);
        }
    }
    search.results.sort_by(|a, b| {
        (a.distance, a.edits.len(), &a.command).cmp(&(b.distance, b.edits.len(), &b.command))
    });
    search.results.truncate(8);
    if search.results.is_empty() {
        unsupported.map_or(Outcome::NoMatch, Outcome::Unsupported)
    } else {
        Outcome::Suggestions(search.results)
    }
}

fn programs<'a>(
    input: &'a str,
    tokens: &[lexer::Token],
    context: &Context,
) -> Result<Vec<(&'a str, usize)>, &'static str> {
    let Some(token) = tokens.first() else {
        return Ok(Vec::new());
    };
    if context.program_shadowed {
        return Ok(Vec::new());
    }
    if !token.literal {
        return Err("Use a literal, unquoted program name.");
    }
    let program = &input[token.range.clone()];
    if program.contains('=') {
        return Err("Environment assignments before commands are not supported yet.");
    }
    if context.program_known && !catalog::PROGRAMS.contains(&program) {
        return Ok(Vec::new());
    }
    Ok(matches(program, catalog::PROGRAMS))
}

#[derive(Clone)]
struct Route {
    path: String,
    next: usize,
    edits: Vec<TextEdit>,
    reasons: Vec<String>,
    distance: usize,
}

struct Search<'a> {
    input: &'a str,
    tokens: &'a [lexer::Token],
    context: &'a Context,
    results: Vec<Suggestion>,
}

impl Search<'_> {
    fn finish(&mut self, route: Route) {
        if !route.edits.is_empty() {
            self.results.push(candidate(
                self.input,
                route.edits,
                route.reasons,
                route.distance,
            ));
        }
    }

    fn walk(&mut self, route: Route) -> Result<(), &'static str> {
        let mut children = catalog::children(&route.path);
        if children.is_empty() {
            self.finish(route);
            return Ok(());
        }
        let position = syntax::command(self.input, self.tokens, route.next, &route.path)?;
        let Some(index) = position.index else {
            self.finish(route);
            return Ok(());
        };
        let token = &self.tokens[index];
        if !token.literal {
            return Err("Use a literal, unquoted subcommand.");
        }
        let word = &self.input[token.range.clone()];
        let stack_available =
            self.context.gh_stack && !self.context.gh_aliases.iter().any(|name| name == "stack");
        let protected = match route.path.as_str() {
            "git" => self.context.git_aliases.iter().any(|alias| alias == word),
            "gh" => {
                self.context.gh_aliases.iter().any(|alias| alias == word)
                    || self
                        .context
                        .gh_extensions
                        .iter()
                        .any(|ext| ext == word && !(ext == "stack" && stack_available))
            }
            _ => false,
        };
        if protected {
            self.finish(route);
            return Ok(());
        }
        if route.path == "gh" && !stack_available {
            // Keep an explicit stack invocation opaque even when absent. It
            // must not be "repaired" into some other built-in command.
            if word == "stack" {
                return Ok(());
            }
            children.retain(|word| *word != "stack");
        }
        let canonical = catalog::canonical(&route.path, word);
        let alternatives = if canonical != word {
            vec![(canonical, 0)]
        } else {
            matches(word, &children)
        };
        let mut unsupported = None;
        for (fixed, distance) in alternatives {
            let mut next = route.clone();
            if distance > 0 {
                next.edits.push(TextEdit {
                    range: token.range.clone(),
                    replacement: fixed.to_owned(),
                });
                next.reasons
                    .push(format!("{} subcommand: {word} → {fixed}", route.path));
            }
            next.path = format!("{} {fixed}", route.path);
            next.next = index + 1;
            next.distance += distance;
            if let Err(message) = self.walk(next) {
                unsupported = Some(message);
            }
        }
        unsupported.map_or(Ok(()), Err)
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
