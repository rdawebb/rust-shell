use crate::commands::Command;
use crate::io_handler::{Redirects, RedirectMode};
use std::io;

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, input: &str) -> io::Result<Option<Command>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        
        let (tokens, redirects) = parse_with_redirects(trimmed);
        
        if tokens.is_empty() {
            return Ok(None);
        }
        
        Command::from_tokens(tokens, redirects)
    }
}

/// Tokenize input and extract redirections
fn parse_with_redirects(input: &str) -> (Vec<String>, Redirects) {
    let tokens = tokenize(input);
    let mut redirects = Redirects::new();
    let mut cmd_tokens = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        match tokens[i].as_str() {
            ">" | "1>" if i + 1 < tokens.len() => {
                redirects = redirects.with_stdout(tokens[i + 1].clone(), RedirectMode::Overwrite);
                i += 2;
            }
            ">>" | "1>>" if i + 1 < tokens.len() => {
                redirects = redirects.with_stdout(tokens[i + 1].clone(), RedirectMode::Append);
                i += 2;
            }
            "2>" if i + 1 < tokens.len() => {
                redirects = redirects.with_stderr(tokens[i + 1].clone(), RedirectMode::Overwrite);
                i += 2;
            }
            "2>>" if i + 1 < tokens.len() => {
                redirects = redirects.with_stderr(tokens[i + 1].clone(), RedirectMode::Append);
                i += 2;
            }
            _ => {
                cmd_tokens.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    
    (cmd_tokens, redirects)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    
    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }

            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }

            '>' if !in_single_quotes && !in_double_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                
                }
                let mut op = String::from(">");
                if chars.peek() == Some(&'>') {
                    op.push(chars.next().unwrap());
                }
                result.push(op);
            }

            '0'..='9' if !in_single_quotes && !in_double_quotes => {
                if chars.peek() == Some(&'>') {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                    let mut op = c.to_string();
                    op.push(chars.next().unwrap());
                    if chars.peek() == Some(&'>') {
                        op.push(chars.next().unwrap());
                    }
                    result.push(op);
                } else {
                    current.push(c);
                }
            }

            ' ' if !in_single_quotes && !in_double_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }

            '\\' if !in_single_quotes => {
                if in_double_quotes {
                    if let Some(&next_c) = chars.peek() {
                        if next_c == '"' || next_c == '\\' {
                            chars.next();
                            current.push(next_c);
                        } else {
                            current.push(c);
                        }
                    } else {
                        current.push(c);
                    }
                } else if let Some(next_c) = chars.next() {
                    current.push(next_c);
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// Strip outer quotes if matching
pub fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        if (bytes[0] == b'\'' && bytes[bytes.len()-1] == b'\'') ||
           (bytes[0] == b'"' && bytes[bytes.len()-1] == b'"') {
            return &s[1..s.len()-1];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_simple() {
        assert_eq!(tokenize("echo hello world"), vec!["echo", "hello", "world"]);
    }
    
    #[test]
    fn test_tokenize_quotes() {
        assert_eq!(tokenize(r#"echo "hello world""#), vec!["echo", "hello world"]);
    }
    
    #[test]
    fn test_tokenize_escaped() {
        assert_eq!(tokenize(r#"echo \"hello\""#), vec!["echo", "\"hello\""]);
    }
    
    #[test]
    fn test_tokenize_append_operator() {
        let tokens = tokenize("echo hello >> out.txt");
        assert_eq!(tokens, vec!["echo", "hello", ">>", "out.txt"]);
    }
    
    #[test]
    fn test_tokenize_stderr_append() {
        let tokens = tokenize("cmd 2>> err.txt");
        assert_eq!(tokens, vec!["cmd", "2>>", "err.txt"]);
    }
    
    #[test]
    fn test_tokenize_mixed_redirects() {
        let tokens = tokenize("cmd > out.txt 2>> err.txt");
        assert_eq!(tokens, vec!["cmd", ">", "out.txt", "2>>", "err.txt"]);
    }
    
    #[test]
    fn test_parse_with_redirects_overwrite() {
        let (tokens, redirects) = parse_with_redirects("echo hello > out.txt 2> err.txt");
        assert_eq!(tokens, vec!["echo", "hello"]);
        assert_eq!(redirects.stdout, Some(("out.txt".to_string(), RedirectMode::Overwrite)));
        assert_eq!(redirects.stderr, Some(("err.txt".to_string(), RedirectMode::Overwrite)));
    }
    
    #[test]
    fn test_parse_with_redirects_append() {
        let (tokens, redirects) = parse_with_redirects("echo hello >> out.txt 2>> err.txt");
        assert_eq!(tokens, vec!["echo", "hello"]);
        assert_eq!(redirects.stdout, Some(("out.txt".to_string(), RedirectMode::Append)));
        assert_eq!(redirects.stderr, Some(("err.txt".to_string(), RedirectMode::Append)));
    }
    
    #[test]
    fn test_parse_with_redirects_mixed() {
        let (tokens, redirects) = parse_with_redirects("echo hello > out.txt 2>> err.txt");
        assert_eq!(tokens, vec!["echo", "hello"]);
        assert_eq!(redirects.stdout, Some(("out.txt".to_string(), RedirectMode::Overwrite)));
        assert_eq!(redirects.stderr, Some(("err.txt".to_string(), RedirectMode::Append)));
    }
    
    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("'hello'"), "hello");
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("hello"), "hello");
        assert_eq!(strip_quotes("'hello"), "'hello");
    }
}
