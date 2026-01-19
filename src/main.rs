use std::path::PathBuf;
use std::time::Instant;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::FileHistory;

mod commands;
mod parser;
mod io_handler;
mod completion;

use completion::ShellCompleter;
use io_handler::IoContext;
use parser::Parser;

const PROMPT: &str = "$ ";

fn run_repl() -> rustyline::Result<()> {
    let parser = Parser::new();

    let mut rl: Editor<ShellCompleter, FileHistory> = Editor::new()?;
    rl.set_helper(Some(ShellCompleter::new()));
    rl.set_bell_style(rustyline::config::BellStyle::Audible);

    let history_file: Option<PathBuf> = dirs::home_dir()
        .map(|mut path| {
            path.push(".shell_history");
            path
        });

    if let Some(ref path) = history_file {
        let _ = rl.load_history(path);
    }

    loop {
        let readline = rl.readline(PROMPT);

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);

                match parser.parse(&line) {
                    Ok(Some(command)) => {
                        if command.is_exit() {
                            break;
                        }
                        
                        match IoContext::new(command.redirects()) {
                            Ok(io_ctx) => {
                                let start = Instant::now();
                                let result = command.execute(io_ctx);
                                let duration = start.elapsed();
                                
                                println!("Command executed in {:.5} s", duration.as_secs_f64());
                                
                                if let Err(e) = result {
                                    eprintln!("Error: {}", e);
                                }
                            }
                            Err(e) => eprintln!("Redirection error: {}", e),
                        }
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    if let Some(path) = history_file {
        let _ = rl.save_history(&path);
    }
    
    Ok(())
}

fn main() {
    run_repl().unwrap_or_else(|e| {
        eprintln!("Shell error: {}", e);
        std::process::exit(1);
    });
}
