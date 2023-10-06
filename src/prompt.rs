use crate::eval::*;

use rustyline::error::ReadlineError;

pub fn run() {
    let mut state = State::new();
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    println!(
        r#"
        ██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
        ██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
        ██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
        ██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
        ██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
        "#
    );

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                if !line.is_empty() {
                    let _ = rl.add_history_entry(&line);
                    if let Err(e) = eval(&line, &mut state) {
                        eprintln!("*** {:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
