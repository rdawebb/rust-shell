use crate::io_handler::IoContext;
use std::env;
use std::io;

pub fn execute(io_ctx: &mut IoContext) -> io::Result<()> {
    match env::current_dir() {
        Ok(pwd) => io_ctx.write_stdout(&pwd.display().to_string()),
        Err(e) => io_ctx.write_stderr(&format!("pwd: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_handler::Redirects;
    
    #[test]
    fn test_pwd() {
        let redirects = Redirects::new();
        let mut io_ctx = IoContext::new(&redirects).unwrap();
        
        assert!(execute(&mut io_ctx).is_ok());
    }
}
