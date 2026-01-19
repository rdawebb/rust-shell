use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Get all builtin commands
pub fn builtin_commands() -> Vec<String> {
    vec![
        "echo".to_string(),
        "exit".to_string(),
        "type".to_string(),
        "pwd".to_string(),
        "cd".to_string(),
    ]
}

/// Get all executables from PATH
pub fn path_commands() -> Vec<String> {
    let mut commands = Vec::new();
    
    if let Some(path_env) = env::var_os("PATH") {
        for dir in env::split_paths(&path_env) {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && is_executable(&entry.path()) {
                            if let Some(name) = entry.file_name().to_str() {
                                commands.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Remove duplicates and sort
    commands.sort();
    commands.dedup();
    commands
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_commands() {
        let builtins = builtin_commands();
        assert!(builtins.contains(&"echo".to_string()));
        assert!(builtins.contains(&"exit".to_string()));
    }
    
    #[test]
    fn test_path_commands() {
        let commands = path_commands();
        // Should contain common commands
        assert!(commands.contains(&"ls".to_string()));
        // Should not contain duplicates
        let unique_count = commands.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(commands.len(), unique_count);
    }
}
