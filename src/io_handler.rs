use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub struct IoContext {
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
}

impl IoContext {
    pub fn new(redirects: &Redirects) -> io::Result<Self> {
        let stdout: Box<dyn Write> = match &redirects.stdout {
            Some((path, RedirectMode::Append)) => Box::new(open_file(path, true)?),
            Some((path, RedirectMode::Overwrite)) => Box::new(open_file(path, false)?),
            None => Box::new(io::stdout()),
        };
        
        let stderr: Box<dyn Write> = match &redirects.stderr {
            Some((path, RedirectMode::Append)) => Box::new(open_file(path, true)?),
            Some((path, RedirectMode::Overwrite)) => Box::new(open_file(path, false)?),
            None => Box::new(io::stderr()),
        };
        
        Ok(Self { stdout, stderr })
    }
    
    pub fn write_stdout(&mut self, msg: &str) -> io::Result<()> {
        writeln!(self.stdout, "{}", msg)
    }
    
    pub fn write_stderr(&mut self, msg: &str) -> io::Result<()> {
        writeln!(self.stderr, "{}", msg)
    }
    
    // pub fn stdout(&mut self) -> &mut dyn Write {
    //     &mut *self.stdout
    // }

    // pub fn stderr(&mut self) -> &mut dyn Write {
    //     &mut *self.stderr
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectMode {
    Overwrite,
    Append,
}

impl Default for RedirectMode {
    fn default() -> Self {
        RedirectMode::Overwrite
    }
}

/// Stdout/stderr redirections
#[derive(Debug, Clone, Default)]
pub struct Redirects {
    pub stdout: Option<(String, RedirectMode)>,
    pub stderr: Option<(String, RedirectMode)>,
}

impl Redirects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stdout(mut self, path: String, mode: RedirectMode) -> Self {
        self.stdout = Some((path, mode));
        self
    }

    pub fn with_stderr(mut self, path: String, mode: RedirectMode) -> Self {
        self.stderr = Some((path, mode));
        self
    }
}

pub fn open_file(path: &str, append: bool) -> io::Result<File> {
    let path = Path::new(path.trim());

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    if append {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    } else {
        File::create(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_open_file_overwrite() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_redirect/output.txt");
        
        let result = open_file(test_path.to_str().unwrap(), false);
        assert!(result.is_ok());
        assert!(test_path.exists());
        
        // Cleanup
        let _ = fs::remove_dir_all(temp_dir.join("test_redirect"));
    }

    #[test]
    fn test_open_file_append() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_append.txt");
        
        // Write first line
        {
            let mut file = open_file(test_path.to_str().unwrap(), false).unwrap();
            writeln!(file, "first").unwrap();
        }
        
        // Append second line
        {
            let mut file = open_file(test_path.to_str().unwrap(), true).unwrap();
            writeln!(file, "second").unwrap();
        }
        
        // Verify both lines exist
        let content = fs::read_to_string(&test_path).unwrap();
        assert_eq!(content, "first\nsecond\n");
        
        // Cleanup
        let _ = fs::remove_file(test_path);
    }
    
    #[test]
    fn test_redirect_mode_enum() {
        let redirects = Redirects::new()
            .with_stdout("out.txt".to_string(), RedirectMode::Append)
            .with_stderr("err.txt".to_string(), RedirectMode::Overwrite);
        
        assert_eq!(redirects.stdout, Some(("out.txt".to_string(), RedirectMode::Append)));
        assert_eq!(redirects.stderr, Some(("err.txt".to_string(), RedirectMode::Overwrite)));
    }
}
