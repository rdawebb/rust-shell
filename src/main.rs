#[allow(unused_imports)]
use std::io::{self, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::env;
use std::path::Path;
//use std::time::Instant;

enum BuiltinCommand {
    Echo(String),
    Type(String),
    Exit,
    External(String, Vec<String>),
    Pwd,
    Cd(String),
}

// Built-in command names
const CMD_ECHO: &str = "echo";
const CMD_TYPE: &str = "type";
const CMD_EXIT: &str = "exit";
const CMD_PWD: &str = "pwd";
const CMD_CD: &str = "cd";

const PROMPT: &str = "$ ";

fn find_executable(cmd: &str) -> Option<std::path::PathBuf> {
    if let Some(path_env) = std::env::var_os("PATH") {
        std::env::split_paths(&path_env)
            .filter_map(|dir| {
                let path = dir.join(cmd);
                let abs_path = if path.is_absolute() {
                    path
                } else {
                    std::env::current_dir().unwrap().join(path)
                };
                if abs_path.is_file() && is_executable(&abs_path) {
                    Some(abs_path)
                } else {
                    None
                }
            })
            .next()
    } else {
        None
    }
}

fn is_executable(path: &std::path::Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        return permissions.mode() & 0o111 != 0;
    }
    false
}

fn parse_arguments(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = args.chars().peekable();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                if in_double_quotes {
                    current.push(c);
                } else {
                    in_single_quotes = !in_single_quotes;
                }
                continue;
            }
            '"' => {
                if in_single_quotes {
                    current.push(c);
                } else {
                    in_double_quotes = !in_double_quotes;
                }
                continue;
            }
            ' ' if !in_single_quotes && !in_double_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            '\\' if !(in_single_quotes || in_double_quotes) => {
                if let Some(next_c) = chars.next() {
                    current.push(next_c);
                } else {
                    current.push(c);
                }
                continue;
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

fn parse_command(input: &str) -> Option<BuiltinCommand> {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return None;
    }
    
    if trimmed == CMD_EXIT {
        return Some(BuiltinCommand::Exit);
    }
    
    if trimmed.starts_with(&format!("{} ", CMD_ECHO)) {
        let args = &trimmed[CMD_ECHO.len() + 1..];
        let parsed_args = parse_arguments(args);
        let message = parsed_args.join(" ");
        return Some(BuiltinCommand::Echo(message));
    }
    
    if trimmed.starts_with(&format!("{} ", CMD_TYPE)) {
        let arg = trimmed[CMD_TYPE.len() + 1..].to_string();
        return Some(BuiltinCommand::Type(arg));
    }

    if trimmed == CMD_PWD {
        return Some(BuiltinCommand::Pwd);
    }

    if trimmed.starts_with(&format!("{} ", CMD_CD)) {
        let arg = trimmed[CMD_CD.len() + 1..].to_string();
        return Some(BuiltinCommand::Cd(arg));
    }

    // Parse external command
    let parts = parse_arguments(trimmed);
    if !parts.is_empty() {
        let cmd = parts[0].clone();
        let args = parts[1..].to_vec();
        return Some(BuiltinCommand::External(cmd, args));
    }
    None
}

fn cmd_echo(message: &str) {
    println!("{}", message);
}

fn cmd_type(arg: &str) -> bool {
    if arg == CMD_ECHO || arg == CMD_EXIT || arg == CMD_TYPE || arg == CMD_PWD || arg == CMD_CD {
        println!("{} is a shell builtin", arg);
        true
    } else if let Some(path) = find_executable(arg) {
        println!("{} is {}", arg, path.display());
        true
    } else {
        println!("{}: not found", arg);
        false
    }
}

fn execute_external_command(cmd: &str, args: &[String]) -> bool {
    if let Some(_exec_path) = find_executable(cmd) {
        match Command::new(cmd).args(args).status() {
            Ok(_status) => true,
            Err(e) => {
                eprintln!("Failed to execute {}: {}", cmd, e);
                false
            }
        }
    } else {
        println!("{}: command not found", cmd);
        false
    }
}

fn execute_command(cmd: BuiltinCommand) -> bool {
    match cmd {
        BuiltinCommand::Echo(message) => {
            cmd_echo(&message);
            true
        }
        BuiltinCommand::Type(arg) => cmd_type(&arg),
        BuiltinCommand::Exit => false, // Signal to exit
        BuiltinCommand::External(cmd_name, args) => execute_external_command(&cmd_name, &args),
        BuiltinCommand::Pwd => {
            cmd_pwd();
            true
        }
        BuiltinCommand::Cd(arg) => {
            cmd_cd(&arg);
            true
        }
    }
}

fn cmd_pwd() {
    if let Ok(pwd) = env::current_dir() {
        println!("{}", pwd.display());
    } else {
        eprintln!("Failed to get current directory");
    }
}

fn cmd_cd(arg: &str) {
    let path = Path::new(arg);
    if path == "~" {
        if let Some(home) = env::home_dir() {
            env::set_current_dir(home).unwrap();
        }
    } else if path.exists() {
        if let Err(e) = env::set_current_dir(&path) {
            eprintln!("cd {}: {}", arg, e);
        }
    } else {
        eprintln!("cd: {}: No such file or directory", arg);
    }
}

fn run_repl() {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        // Read the command from the user
        let mut command = String::new();
        if io::stdin().read_line(&mut command).unwrap() == 0 {
            break;
        }

        // Parse the command
        if let Some(cmd) = parse_command(&command) {
            // Execute the command and check for exit
            match cmd {
                BuiltinCommand::Exit => break,
                other => {
                    //let now = Instant::now();
                    execute_command(other);
                    //let elapsed = now.elapsed();
                    //println!("Command executed in {:.5}s", elapsed.as_secs_f64());
                }
            }
        }
    }
}

fn main() {
    run_repl();
}
