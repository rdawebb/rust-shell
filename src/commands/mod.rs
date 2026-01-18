pub mod builtin;

use crate::io_handler::{IoContext, Redirects, RedirectMode, open_file};
use crate::parser::strip_quotes;
use std::env;
use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub enum Command {
    Echo { args: Vec<String>, redirects: Redirects },
    Type { arg: String, redirects: Redirects },
    Pwd { redirects: Redirects },
    Cd { path: String, redirects: Redirects },
    Exit { redirects: Redirects },
    External { name: String, args: Vec<String>, redirects: Redirects },
}

impl Command {
    pub fn from_tokens(tokens: Vec<String>, redirects: Redirects) -> io::Result<Option<Self>> {
        if tokens.is_empty() {
            return Ok(None);
        }
        
        let cmd = &tokens[0];
        let args: Vec<String> = tokens[1..].iter().map(|s| strip_quotes(s).to_string()).collect();
        
        let command = match cmd.as_str() {
            "exit" => Self::Exit { redirects },
            "echo" => Self::Echo { args, redirects },
            "type" => {
                if args.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "type: missing argument"));
                }
                Self::Type { arg: args[0].clone(), redirects }
            }
            "pwd" => Self::Pwd { redirects },
            "cd" => {
                if args.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "cd: missing argument"));
                }
                Self::Cd { path: args[0].clone(), redirects }
            }
            _ => {
                let name = strip_quotes(cmd).to_string();
                Self::External { name, args, redirects }
            }
        };
        
        Ok(Some(command))
    }
    
    pub fn is_exit(&self) -> bool {
        matches!(self, Self::Exit { .. })
    }
    
    pub fn redirects(&self) -> &Redirects {
        match self {
            Self::Echo { redirects, .. } |
            Self::Type { redirects, .. } |
            Self::Pwd { redirects, .. } |
            Self::Cd { redirects, .. } |
            Self::Exit { redirects, .. } |
            Self::External { redirects, .. } => redirects,
        }
    }
    
    pub fn execute(&self, mut io_ctx: IoContext) -> io::Result<()> {
        match self {
            Self::Echo { args, .. } => {
                builtin::echo_cmd::execute(args, &mut io_ctx)
            }
            Self::Type { arg, .. } => {
                builtin::type_cmd::execute(arg, &mut io_ctx)
            }
            Self::Pwd { .. } => {
                builtin::pwd_cmd::execute(&mut io_ctx)
            }
            Self::Cd { path, .. } => {
                builtin::cd_cmd::execute(path, &mut io_ctx)
            }
            Self::External { name, args, redirects } => {
                execute_external(name, args, redirects)
            }
            Self::Exit { .. } => Ok(()),
        }
    }
}

fn execute_external(
    name: &str,
    args: &[String],
    redirects: &Redirects,
) -> io::Result<()> {
    let exec_path = match find_executable(name) {
        Some(path) => path,
        None => {
            eprintln!("{}: command not found", name);
            return Ok(());
        }
    };

    let mut command = ProcessCommand::new(&exec_path);
    command.arg0(name);
    command.args(args);
    
    // Handle stdout redirection
    if let Some((path, mode)) = &redirects.stdout {
        let append = matches!(mode, RedirectMode::Append);
        let file = open_file(path, append)?;
        command.stdout(Stdio::from(file));
    }
    
    // Handle stderr redirection
    if let Some((path, mode)) = &redirects.stderr {
        let append = matches!(mode, RedirectMode::Append);
        let file = open_file(path, append)?;
        command.stderr(Stdio::from(file));
    }
    
    match command.status() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to execute {}: {}", name, e);
            Ok(())
        }
    }
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path_env| {
        env::split_paths(&path_env)
            .filter_map(|dir| {
                let path = dir.join(cmd);
                let abs_path = if path.is_absolute() {
                    path.clone()
                } else {
                    env::current_dir().ok()?.join(&path)
                };
                
                if abs_path.is_file() && is_executable(&abs_path) {
                    return Some(abs_path);
                }

                // Try common executable extensions
                #[cfg(windows)]
                {
                    for ext in &[".exe", ".bat", ".cmd", ".com"] {
                        if !cmd.ends_with(ext) {
                            let path_with_ext = dir.join(format!("{}{}", cmd, ext));
                            let abs_path = if path_with_ext.is_absolute() {
                                path_with_ext
                            } else {
                                env::current_dir().ok()?.join(path_with_ext)
                            };
                            
                            if abs_path.is_file() && is_executable(&abs_path) {
                                return Some(abs_path);
                            }
                        }
                    }
                }
                
                None
            })
            .next()
    })
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        // Check if it's a file in PATH
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_executable() {
        // Should find common Unix commands
        assert!(find_executable("ls").is_some());
        assert!(find_executable("nonexistent_command_xyz").is_none());
    }
}
