#[allow(unused_imports)]
use std::io::{self, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::process::Stdio;
use std::env;
use std::path::Path;

enum BuiltinCommand {
    Echo(String, Option<String>, Option<String>),
    Type(String, Option<String>, Option<String>),
    Exit,
    External(String, Vec<String>, Option<String>, Option<String>),
    Pwd(Option<String>, Option<String>),
    Cd(String, Option<String>, Option<String>),
}

enum ShellStream {
    Stdout,
    Stderr,
}

// Built-in command names
const CMD_ECHO: &str = "echo";
const CMD_TYPE: &str = "type";
const CMD_EXIT: &str = "exit";
const CMD_PWD: &str = "pwd";
const CMD_CD: &str = "cd";

const PROMPT: &str = "$ ";


fn shell_write(file: &Option<String>, msg: &str, stream: ShellStream) {
    if let Some(path) = file {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = writeln!(f, "{}", msg);
        }
    } else {
        match stream {
            ShellStream::Stdout => println!("{}", msg),
            ShellStream::Stderr => eprintln!("{}", msg),
        }
    }
}

fn strip_outer_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        if (bytes[0] == b'\'' && bytes[bytes.len()-1] == b'\'') ||
           (bytes[0] == b'"' && bytes[bytes.len()-1] == b'"') {
            return &s[1..s.len()-1];
        }
    }
    s
}

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

fn split_redirection(input: &str) -> (Vec<String>, Option<String>, Option<String>) {
    let tokens = parse_arguments(input);
    let mut out_file = None;
    let mut err_file = None;
    let mut cmd_tokens = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i] == ">" || tokens[i] == "1>" {
            if i + 1 < tokens.len() {
                out_file = Some(tokens[i + 1].clone());
            }
            break; // No filename provided, ignore
        } else if tokens[i] == "2>" {
            if i + 1 < tokens.len() {
                err_file = Some(tokens[i + 1].clone());
            }
            break; // No filename provided, ignore
        } else {
            cmd_tokens.push(tokens[i].clone());
        }
        i += 1;
    }
    (cmd_tokens, out_file, err_file)
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
            '\\' => {
                if in_single_quotes {
                    current.push(c);
                } else if in_double_quotes {
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
                } else {
                    if let Some(next_c) = chars.next() {
                        current.push(next_c);
                    }
                }
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
    let (cmd_tokens, out_file, err_file) = split_redirection(trimmed);

    if cmd_tokens.is_empty() {
        return None;
    }

    if trimmed == CMD_EXIT {
        return Some(BuiltinCommand::Exit);
    }
    
    if cmd_tokens.len() > 0 && cmd_tokens[0] == CMD_ECHO {
        let message = cmd_tokens[1..].join(" ");
        return Some(BuiltinCommand::Echo(message, out_file, err_file));
    }
    
    if cmd_tokens.len() > 1 && cmd_tokens[0] == CMD_TYPE {
        let arg = cmd_tokens[1].clone();
        return Some(BuiltinCommand::Type(arg, out_file, err_file));
    }

    if trimmed == CMD_PWD {
        return Some(BuiltinCommand::Pwd(out_file, err_file));
    }

    if cmd_tokens.len() > 1 && cmd_tokens[0] == CMD_CD {
        let arg = cmd_tokens[1].clone();
        return Some(BuiltinCommand::Cd(arg, out_file, err_file));
    }

    // Parse external command
    let cmd = strip_outer_quotes(&cmd_tokens[0]).to_string();
    let args = cmd_tokens[1..].iter().map(|a| strip_outer_quotes(a).to_string()).collect::<Vec<_>>();
    Some(BuiltinCommand::External(cmd, args, out_file, err_file))
}

fn cmd_echo(message: &str, out_file: &Option<String>, err_file: &Option<String>) {
    if let Some(file) = out_file {
        let _ = std::fs::File::create(file.trim());
    }
    if let Some(file) = err_file {
        let _ = std::fs::File::create(file.trim());
    }
    
    if let Some(file) = out_file {
        let file = file.trim();
        if let Some(parent) = std::path::Path::new(file).parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    shell_write(&err_file, &format!("Failed to create directory {}: {}", parent.display(), e), ShellStream::Stderr);
                }
            }
        }
        if let Ok(mut f) = std::fs::File::create(file) {
            let _ = writeln!(f, "{}", message);
        } else {
            shell_write(&err_file, &format!("Failed to write to file: {}", file), ShellStream::Stderr);
        }
    } else {
        shell_write(&out_file, &message, ShellStream::Stdout);
    }
}

fn cmd_type(arg: &str, out_file: &Option<String>, err_file: &Option<String>) -> bool {
    if let Some(file) = out_file {
        let _ = std::fs::File::create(file.trim());
    }
    if let Some(file) = err_file {
        let _ = std::fs::File::create(file.trim());
    }

    if arg == CMD_ECHO || arg == CMD_EXIT || arg == CMD_TYPE || arg == CMD_PWD || arg == CMD_CD {
        shell_write(&out_file, &format!("{} is a shell builtin", arg), ShellStream::Stdout);
        true
    } else if let Some(path) = find_executable(arg) {
        shell_write(&out_file, &format!("{} is {}", arg, path.display()), ShellStream::Stdout);
        true
    } else {
        shell_write(&err_file, &format!("{}: not found", arg), ShellStream::Stderr);
        false
    }
}

fn execute_external_command(cmd: &str, args: &[String], out_file: &Option<String>, err_file: &Option<String>) -> bool {
    if let Some(_exec_path) = find_executable(cmd) {
        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(file) = out_file {
            let file = file.trim();
            let path = std::path::Path::new(file);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        shell_write(&err_file, &format!("Failed to create directory {}: {}", parent.display(), e), ShellStream::Stderr);
                    }
                }
            }
            match std::fs::File::create(file) {
                Ok(f) => {
                    command.stdout(Stdio::from(f));
                }
                Err(e) => {
                    shell_write(&err_file, &format!("Failed to open file: {} {}", file, e), ShellStream::Stderr);
                    return false;
                }
            }
        }
        if let Some(file) = err_file {
            let file = file.trim();
            let path = std::path::Path::new(file);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        shell_write(&err_file, &format!("Failed to create directory {}: {}", parent.display(), e), ShellStream::Stderr);
                    }
                }
            }
            match std::fs::File::create(file) {
                Ok(f) => {
                    command.stderr(Stdio::from(f));
                }
                Err(e) => {
                    shell_write(&err_file, &format!("Failed to open file: {} {}", file, e), ShellStream::Stderr);
                    return false;
                }
            }
        }
        match command.status() {
            Ok(_status) => true,
            Err(e) => {
                shell_write(&err_file, &format!("Failed to execute {}: {}", cmd, e), ShellStream::Stderr);
                false
            }
        }
    } else {
        shell_write(&err_file, &format!("{}: command not found", cmd), ShellStream::Stderr);
        false
    }
}

fn execute_command(cmd: BuiltinCommand) -> bool {
    match cmd {
        BuiltinCommand::Echo(message, out_file, err_file) => {
            cmd_echo(&message, &out_file, &err_file);
            true
        }
        BuiltinCommand::Type(arg, out_file, err_file) => cmd_type(&arg, &out_file, &err_file),
        BuiltinCommand::Exit => false, // Signal to exit
        BuiltinCommand::External(cmd_name, args, out_file, err_file) => execute_external_command(&cmd_name, &args, &out_file, &err_file),
        BuiltinCommand::Pwd(out_file, err_file) => {
            cmd_pwd(&out_file, &err_file);
            true
        }
        BuiltinCommand::Cd(arg, _out_file, err_file) => {
            cmd_cd(&arg, &err_file);
            true
        }
    }
}

fn cmd_pwd(out_file: &Option<String>, err_file: &Option<String>) {
    if let Some(file) = out_file {
        let _ = std::fs::File::create(file.trim());
    }
    if let Some(file) = err_file {
        let _ = std::fs::File::create(file.trim());
    }

    if let Ok(pwd) = env::current_dir() {
        shell_write(&out_file, &format!("{}", pwd.display()), ShellStream::Stdout);
    } else {
        shell_write(&err_file, "Failed to get current directory", ShellStream::Stderr);
    }
}

fn cmd_cd(arg: &str, err_file: &Option<String>) {
    if let Some(file) = err_file {
        let _ = std::fs::File::create(file.trim());
    }

    let path = Path::new(arg);
    if arg == "~" {
        if let Ok(home) = std::env::var("HOME") {
            if let Err(e) = env::set_current_dir(&home) {
                shell_write(&err_file, &format!("cd {}: {}", arg, e), ShellStream::Stderr);
            }
        } else {
            shell_write(&err_file, "cd: HOME not set", ShellStream::Stderr);
        }
    } else if path.exists() {
        if let Err(e) = env::set_current_dir(&path) {
            shell_write(&err_file, &format!("cd {}: {}", arg, e), ShellStream::Stderr);
        }
    } else {
        shell_write(&err_file, &format!("cd: {}: No such file or directory", arg), ShellStream::Stderr);
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
