#[allow(unused_imports)]
use std::io::{self, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

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

fn main() {
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read the command from the user
        let mut command = String::new();
        if io::stdin().read_line(&mut command).unwrap() == 0 {
            break;
        }

        // Trim whitespace and check for empty input
        let trimmed = command.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Handle exit command
        if trimmed == "exit" {
            break;
        }

        // Handle echo command
        if trimmed.starts_with("echo ") {
            let message = &trimmed[5..];
            println!("{}", message);
            continue;
        }

        // Handle type command
        if trimmed.starts_with("type ") {
            let arg = &trimmed[5..];
            if arg == "echo" || arg == "exit" || arg == "type" {
                println!("{} is a shell builtin", arg);
            } else {
                if let Some(path) = find_executable(arg) {
                    println!("{} is {}", arg, path.display());
                } else {
                    println!("{}: not found", arg);
                }
            }
            continue;
        }

        // Else, get the command and arguments
        let mut parts = trimmed.split_whitespace();
        let cmd = parts.next().unwrap();
        let args = parts.collect::<Vec<&str>>();

        // Execute the command
        if let Some(_exec_path) = find_executable(cmd) {
            let status = Command::new(cmd)
                .args(&args)
                .status();
            match status {
                Ok(_status) => {}, // Optionally handle exit code
                Err(e) => eprintln!("Failed to execute {}: {}", cmd, e),
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}
