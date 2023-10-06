use crate::eval::*;

use rustyline::error::ReadlineError;

pub fn run() {
    println!(
        r#"
        ██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
        ██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
        ██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
        ██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
        ██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
        "#
    );

    let mut state = State::new();
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                if !line.is_empty() {
                    let _ = rl.add_history_entry(&line);
                    match eval(&line, &mut state) {
                        Ok(d) => {
                            for data in d {
                                println!("{:?}", data);
                            }
                        }
                        Err(e) => eprintln!("*** {:?}", e),
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
                println!("Unexpected Error: {:?}", err);
                break;
            }
        }
    }
}
