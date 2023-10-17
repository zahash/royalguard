use crate::crypt::*;
use crate::eval::*;
use crate::store::Store;

use anyhow::Context;
use clap::Parser;
use rustyline::error::ReadlineError;

const LOGO: &str = r#"
██████   ██████  ██    ██  █████  ██           ██████  ██    ██  █████  ██████  ██████  
██   ██ ██    ██  ██  ██  ██   ██ ██          ██       ██    ██ ██   ██ ██   ██ ██   ██ 
██████  ██    ██   ████   ███████ ██          ██   ███ ██    ██ ███████ ██████  ██   ██ 
██   ██ ██    ██    ██    ██   ██ ██          ██    ██ ██    ██ ██   ██ ██   ██ ██   ██ 
██   ██  ██████     ██    ██   ██ ███████      ██████   ██████  ██   ██ ██   ██ ██████  
"#;

const HELP: &str = r#"
Add, Update:
    set gmail user = sussolini sensitive pass = 'use single quote for spaces' url = mail.google.sus
    set gmail sensitive pass = updatedpassword user = updated_user

Delete whole record: 
    del gmail

Delete fields: 
    del gmail url pass

Show -- replaces sensitive values with *****:
    show all
    show gmail
    show user is sussolini and (pass contains sus or url matches '.*com')

Show (filter by name):
    show . contains gmail

Reveal -- works exactly like Show but does not respect sensitivity
    reveal user is sussolini and (pass contains sus or url matches '.*com')

History -- show changes made overtime:
    history gmail
    reveal history gmail

Rename:
    rename gmail gmail2

Copy field to clipboard:
    copy gmail pass

Import:
    import 'path/to/file.txt'

Importing requires the below data format. Each line being a new record
'gmail' user = 'joseph ballin' sensitive pass = 'ни шагу назад, товарищи!'
'discord' user = 'pablo susscobar' pass = 'plata o plomo'

Change Master Password: chmpw
"#;

/// Royal Guard
#[derive(Parser)]
struct Cli {
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

fn save(fpath: &str, master_pass: &str, store: &Store) {
    println!("saving to '{}' ...", fpath);
    match dump(fpath, master_pass, store) {
        Ok(_) => println!("saved successfully!"),
        Err(e) => eprintln!("!! error while saving: {:?}", e),
    }
}

pub fn run() -> anyhow::Result<()> {
    let fpath = match Cli::parse().fpath {
        Some(f) => f,
        None => default_fpath()?,
    };

    println!(env!("CARGO_PKG_VERSION"));
    println!("All data will be saved to file '{}'", fpath);

    let Ok(mut master_pass) = rpassword::prompt_password("master password: ") else {
        println!("Bye!");
        return Ok(());
    };

    let mut store = load(&fpath, &master_pass)?;
    let mut editor = rustyline::DefaultEditor::new()?;

    println!("{}", LOGO);
    println!(env!("CARGO_PKG_VERSION"));

    println!("type 'help' for usage instructions");
    println!("To Quit, press CTRL-C or CTRL-D or type 'exit' or 'quit' (all updates will be auto saved after quitting)");
    println!("type 'save' to save current updates manually");

    loop {
        match editor.readline("> ").as_deref() {
            Ok("clear") | Ok("cls") => editor.clear_screen()?,
            Ok("help") | Ok("HELP") => println!("{}", HELP),
            Ok("exit") | Ok("quit") => {
                save(&fpath, &master_pass, &store);
                break;
            }
            Ok("save") => save(&fpath, &master_pass, &store),
            Ok("chmpw") => {
                let pw = match rpassword::prompt_password("new master password: ") {
                    Ok(pw) if !pw.trim().is_empty() => pw,
                    _ => {
                        println!("abort!");
                        continue;
                    }
                };

                let pw2 = match rpassword::prompt_password("retype new master password: ") {
                    Ok(pw2) if !pw2.trim().is_empty() => pw2,
                    _ => {
                        println!("abort!");
                        continue;
                    }
                };

                if pw != pw2 {
                    println!("!! passwords didn't match");
                    continue;
                }

                master_pass = pw;
                println!("master password changed successfully!");
            }
            Ok(line) => {
                if !line.is_empty() {
                    editor.add_history_entry(line)?;
                    match eval(line, &mut store) {
                        Ok(eval) => {
                            for line in eval.lines() {
                                println!("{}", line)
                            }
                        }
                        Err(e) => eprintln!("!! {:?}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("CTRL-C");
                save(&fpath, &master_pass, &store);
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                save(&fpath, &master_pass, &store);
                break;
            }
            Err(e) => {
                eprintln!("!! Unexpected Error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
