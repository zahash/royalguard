use crate::{
    // crypt::load, 
    eval::*
};

use rustyline::error::ReadlineError;

use clap::Parser;

// /// Royal Guard
// #[derive(Parser)]
// struct CLI {
//     /// encrypted data filepath
//     #[arg(short, long, default_value_t = String::from("~/royalguard"))]
//     fpath: String,
// }

pub fn run() {
    // let fpath = CLI::parse().fpath;

    // let Ok(master_pass) = rpassword::prompt_password("master password: ") else {
    //     println!("Bye!");
    //     return;
    // };

    // let data = load(fpath, &master_pass);

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
                                println!("{}", data);
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
