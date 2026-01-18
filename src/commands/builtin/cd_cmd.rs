use crate::io_handler::IoContext;
use std::env;
use std::io;
use std::path::Path;

pub fn execute(path: &str, io_ctx: &mut IoContext) -> io::Result<()> {
    let target = expand_tilde(path)?;
    
    if !Path::new(&target).exists() {
        return io_ctx.write_stderr(&format!("cd: {}: No such file or directory", path));
    }
    
    match env::set_current_dir(&target) {
        Ok(_) => Ok(()),
        Err(e) => io_ctx.write_stderr(&format!("cd: {}: {}", path, e)),
    }
}

fn expand_tilde(path: &str) -> io::Result<String> {
    if path == "~" || path.starts_with("~/") {
        let home = env::var("HOME").map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "HOME environment variable not set")
        })?;
        Ok(expand_tilde_with_home(path, &home))
    } else {
        Ok(path.to_string())
    }
}

fn expand_tilde_with_home(path: &str, home: &str) -> String {
    if path == "~" {
        home.to_string()
    } else if path.starts_with("~/") {
        format!("{}{}", home, &path[1..])
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expand_tilde_with_home() {
        assert_eq!(expand_tilde_with_home("~", "/home/testuser"), "/home/testuser");
        assert_eq!(expand_tilde_with_home("~/documents", "/home/testuser"), "/home/testuser/documents");
        assert_eq!(expand_tilde_with_home("/absolute/path", "/home/testuser"), "/absolute/path");
    }
}
