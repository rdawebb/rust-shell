use crate::io_handler::IoContext;
use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const BUILTINS: &[&str] = &["echo", "exit", "type", "pwd", "cd"];

pub fn execute(arg: &str, io_ctx: &mut IoContext) -> io::Result<()> {
    if BUILTINS.contains(&arg) {
        io_ctx.write_stdout(&format!("{} is a shell builtin", arg))
    } else if let Some(path) = find_executable(arg) {
        io_ctx.write_stdout(&format!("{} is {}", arg, path.display()))
    } else {
        io_ctx.write_stderr(&format!("{}: not found", arg))
    }
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path_env| {
        env::split_paths(&path_env)
            .filter_map(|dir| {
                let path = dir.join(cmd);
                let abs_path = if path.is_absolute() {
                    path
                } else {
                    env::current_dir().ok()?.join(path)
                };
                
                if abs_path.is_file() && is_executable(&abs_path) {
                    Some(abs_path)
                } else {
                    None
                }
            })
            .next()
    })
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_handler::Redirects;
    
    #[test]
    fn test_type_builtin() {
        let redirects = Redirects::new();
        let mut io_ctx = IoContext::new(&redirects).unwrap();
        
        assert!(execute("echo", &mut io_ctx).is_ok());
    }
}
