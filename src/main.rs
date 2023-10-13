mod crypt;
mod eval;
mod lex;
mod parse;
mod prompt;
mod data;

fn main() -> anyhow::Result<()> {
    prompt::run()
}
