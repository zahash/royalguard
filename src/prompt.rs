use crate::crypt::*;
use crate::eval::*;

use anyhow::Context;
use clap::Parser;
use rustyline::error::ReadlineError;

/// Royal Guard
#[derive(Parser)]
struct CLI {
    /// encrypted data filepath
    #[arg(short, long)]
    fpath: Option<String>,
}

fn default_fpath() -> anyhow::Result<String> {
    let mut fpath = dirs::home_dir().with_context(
        || "unable to automatically determine home directory. please manually provide a filepath instead.",
    )?;
    fpath.push("royalguard");
    Ok(fpath.to_string_lossy().to_string())
}

pub fn run() -> anyhow::Result<()> {
    let fpath = match CLI::parse().fpath {
        Some(f) => f,
        None => default_fpath()?,
    };

    println!("using file '{}'", fpath);

    let Ok(master_pass) = rpassword::prompt_password("master password: ") else {
        println!("Bye!");
        return Ok(());
    };

    let data = load(&fpath, &master_pass)?;

    let mut state = State::from(data);
    let mut editor = rustyline::DefaultEditor::new()?;

    println!(
        r#"
        ██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
        ██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
        ██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
        ██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
        ██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
        "#
    );

    println!("type 'help' on usage instructions");

    loop {
        match editor.readline("> ") {
            Ok(s) if s == "help" || s == "example" || s == "examples" => {
                println!("set gmail user = sussolini pass = amogus url = mail.google.sus");
                println!("set gmail pass = updatedpotatus");
                println!("del gmail");
                println!("show all");
                println!("show gmail");
                println!("show user is sussolini and (pass contains sus or url matches '.*com')");
            }
            Ok(line) => {
                if !line.is_empty() {
                    editor.add_history_entry(&line)?;
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
