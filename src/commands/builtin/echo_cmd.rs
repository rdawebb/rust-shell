use crate::io_handler::IoContext;
use std::io;

pub fn execute(args: &[String], io_ctx: &mut IoContext) -> io::Result<()> {
    let message = args.join(" ");
    io_ctx.write_stdout(&message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_handler::Redirects;
    
    #[test]
    fn test_echo() {
        let redirects = Redirects::new();
        let mut io_ctx = IoContext::new(&redirects).unwrap();
        
        assert!(execute(&["hello".to_string(), "world".to_string()], &mut io_ctx).is_ok());
    }
}
