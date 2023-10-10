use crate::crypt::*;
use crate::eval::*;

use clap::Parser;
use rustyline::error::ReadlineError;

/// Royal Guard
#[derive(Parser)]
struct CLI {
    /// encrypted data filepath
    #[arg(short, long, default_value_t = String::from("~/royalguard"))]
    fpath: String,
}

pub fn run() -> anyhow::Result<()> {
    let fpath = CLI::parse().fpath;
    let Ok(master_pass) = rpassword::prompt_password("master password: ") else {
        println!("Bye!");
        return Ok(());
    };

    let data = load(&fpath, &master_pass)?;

    println!(
        r#"
        ██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
        ██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
        ██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
        ██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
        ██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
        "#
    );

    let mut state = State::from(data);
    let mut rl = rustyline::DefaultEditor::new()?;

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
                        Err(e) => eprintln!("!! {:?}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                println!("saving to {} ...", &fpath);
                dump(&fpath, &master_pass, state.into())?;
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                println!("saving to {} ...", &fpath);
                dump(&fpath, &master_pass, state.into())?;
                break;
            }
            Err(err) => {
                println!("Unexpected Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
