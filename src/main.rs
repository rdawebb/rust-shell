use std::io::{self, Write};
use std::time::Instant;

mod commands;
mod parser;
mod io_handler;

use io_handler::IoContext;
use parser::Parser;

const PROMPT: &str = "$ ";

fn run_repl() {
    let parser = Parser::new();
    
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            break;
        }

        match parser.parse(&input) {
            Ok(Some(command)) => {
                if command.is_exit() {
                    break;
                }
                
                match IoContext::new(command.redirects()) {
                    Ok(io_ctx) => {
                        if let Err(e) = {
                            let start = Instant::now();
                            let result = command.execute(io_ctx);
                            let duration = start.elapsed();
                            println!("Command executed in {:.5} s", duration.as_secs_f64());
                            result
                        } {
                            eprintln!("Error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Redirection error: {}", e),
                }
            }
            Ok(None) => {} // Empty command
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}

fn main() {
    run_repl();
}
