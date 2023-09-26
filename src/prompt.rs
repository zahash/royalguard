use inquire::Text;

use crate::eval::*;

pub fn run() {
    println!("** PadLock **");

    let mut state = State::new();

    loop {
        let text = Text::new("").prompt().expect("prompt error");
        match eval(&text, &mut state) {
            Ok(_) => {}
            Err(e) => eprintln!("*** {:?}", e),
        }
    }
}
