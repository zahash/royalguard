mod crypt;
mod eval;
mod lex;
mod parse;
mod prompt;
mod store;

// launch prompt. ask for master password

// add name='some name with spaces' user=zahash pass=asdf url='https://asdf.com'

// set 'some name with spaces' user=zahash.z
// set prev user=zahash.z

// show name='some name with spaces' or (name contains asdf and url matches '.+asdf.+')
// show 'some name'
// show all
// show prev

// del 'some name'
// del prev

// history 'some name'
// history prev

fn main() {
    prompt::run().unwrap();
}
