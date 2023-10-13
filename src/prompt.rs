use crate::crypt::*;
use crate::eval::*;

use anyhow::Context;
use clap::Parser;
use rustyline::error::ReadlineError;

const LOGO: &'static str = r#"
██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
"#;

const HELP: &'static str = r#"
Add, Update:
    set gmail user = sussolini sensitive pass = 'use single quote for spaces' url = mail.google.sus
    set gmail sensitive pass = updatedpassword

Delete whole record: 
    del gmail

Delete fields: 
    del gmail url pass

Show -- replaces sensitive values with *****:
    show all
    show gmail
    show user is sussolini and (pass contains sus or url matches '.*com')

Show (filter by name):
    show $name contains mail
    show . contains mail

Reveal -- works exactly like Show but doesn't respect sensitivity
    reveal user is sussolini and (pass contains sus or url matches '.*com')

Copy field to clipboard:
    copy gmail pass
"#;

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

    println!(env!("CARGO_PKG_VERSION"));
    println!("All data will be saved to file '{}'", fpath);

    let Ok(master_pass) = rpassword::prompt_password("master password: ") else {
        println!("Bye!");
        return Ok(());
    };

    let mut store = load(&fpath, &master_pass)?;
    let mut editor = rustyline::DefaultEditor::new()?;

    println!("{}", LOGO);
    println!(env!("CARGO_PKG_VERSION"));

    println!("type 'help' on usage instructions");
    println!("To Quit, press CTRL-C or CTRL-D or type 'exit' or 'quit' (all updates will be auto saved after quitting)");
    println!("type 'save' to save current updates manually");

    loop {
        match editor.readline("> ") {
            Ok(s) if s == "clear" || s == "cls" => editor.clear_screen()?,
            Ok(s) if s == "help" || s == "HELP" => println!("{}", HELP),
            Ok(s) if s == "exit" || s == "quit" => {
                println!("saving to '{}' ...", &fpath);
                dump(&fpath, &master_pass, store)?;
                println!("saved successfully!");
                break;
            }
            Ok(s) if s == "save" => {
                println!("saving to '{}' ...", &fpath);
                dump(&fpath, &master_pass, store.clone())?;
                println!("saved successfully!");
            }
            Ok(line) => {
                if !line.is_empty() {
                    editor.add_history_entry(&line)?;
                    match eval(&line, &mut store) {
                        Ok(records) => {
                            for r in records {
                                println!("{}", r);
                            }
                        }
                        Err(e) => eprintln!("!! {:?}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                println!("saving to '{}' ...", &fpath);
                dump(&fpath, &master_pass, store)?;
                println!("saved successfully!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                println!("saving to '{}' ...", &fpath);
                dump(&fpath, &master_pass, store)?;
                println!("saved successfully!");
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
