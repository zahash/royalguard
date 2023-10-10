mod crypt;
mod eval;
mod lex;
mod parse;
mod prompt;
mod store;

fn main() -> anyhow::Result<()> {
    prompt::run()
}
