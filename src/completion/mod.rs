mod command;

use std::sync::OnceLock;

use command::{builtin_commands, path_commands};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use rustyline::Result as RustylineResult;

static PATH_COMMANDS: OnceLock<Vec<String>> = OnceLock::new();

pub struct ShellCompleter {
    builtins: Vec<String>,
}

impl ShellCompleter {
    pub fn new() -> Self {
        Self {
            builtins: builtin_commands(),
        }
    }

    fn get_all_commands(&self) -> Vec<String> {
        let path_cmds = PATH_COMMANDS.get_or_init(|| {
            path_commands()
        });
        
        let mut all_cmds = self.builtins.clone();
        all_cmds.extend(path_cmds.clone());
        all_cmds.sort();

        all_cmds
    }

    // pub fn add_path_commands(&mut self) {
    // }
}

impl Helper for ShellCompleter {}

impl Hinter for ShellCompleter {
    type Hint = String;
}

impl Highlighter for ShellCompleter {}

impl Validator for ShellCompleter {}

impl Completer for ShellCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos);

        let all_cmds = self.get_all_commands();

        // Find matching commands
        let matches: Vec<Pair> = all_cmds
            .iter()
            .filter(|cmd| cmd.starts_with(word))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: if start == 0 {
                    format!("{} ", cmd)
                } else {
                    cmd.clone()
                },
            })
            .collect();
        
        Ok((start, matches))
    }
}

/// Extract the word being completed and its start position
fn extract_word(line: &str, pos: usize) -> (usize, &str) {
    let line_before_cursor = &line[..pos];
    
    // Find the start of the current word
    let start = line_before_cursor
        .rfind(char::is_whitespace)
        .map(|i| i + 1)
        .unwrap_or(0);
    
    let word = &line_before_cursor[start..];
    (start, word)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_word() {
        assert_eq!(extract_word("echo", 4), (0, "echo"));
        assert_eq!(extract_word("echo ", 5), (5, ""));
        assert_eq!(extract_word("echo hel", 8), (5, "hel"));
        assert_eq!(extract_word("ex", 2), (0, "ex"));
    }
}
