use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Token<'text> {
    Keyword(&'text str),
    Symbol(&'static str),
    Value(&'text str),
}

lazy_static! {
    static ref KEYWORD_REGEX: Regex =
        Regex::new(r#"^(set|del|delete|show|reveal|copy|history|rename|import|secret|sensitive|all|prev|and|or|contains|matches|like|is)\b"#)
            .unwrap();
    static ref VALUE_REGEX: Regex = Regex::new(r#"^([^'\n\s\t\(\)]+|'[^'\n]*')"#).unwrap();
}

#[derive(Debug)]
pub enum LexError {
    InvalidToken { pos: usize },
}

pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    match text.is_empty() {
        true => Ok(vec![]),
        false => {
            let mut tokens = vec![];
            let mut pos = 0;

            loop {
                while let Some(" ") | Some("\n") = text.get(pos..pos + 1) {
                    pos += 1;
                }

                if pos >= text.len() {
                    break;
                }

                let (token, next_pos) = lex_token(text, pos)?;
                tokens.push(token);
                pos = next_pos;
            }

            Ok(tokens)
        }
    }
}

fn lex_token(text: &str, pos: usize) -> Result<(Token, usize), LexError> {
    lex_keyword(text, pos)
        .or(lex_symbol(text, pos, "="))
        .or(lex_symbol(text, pos, "("))
        .or(lex_symbol(text, pos, ")"))
        .or(lex_value(text, pos))
        .ok_or(LexError::InvalidToken { pos })
}

fn lex_keyword(text: &str, pos: usize) -> Option<(Token, usize)> {
    let (token, pos) = lex_with_pattern(text, pos, &KEYWORD_REGEX)?;
    Some((Token::Keyword(token), pos))
}

fn lex_value(text: &str, pos: usize) -> Option<(Token, usize)> {
    let (mut token, pos) = lex_with_pattern(text, pos, &VALUE_REGEX)?;
    if let Some(stripped) = token.strip_prefix("'") {
        token = stripped;
    }
    if let Some(stripped) = token.strip_suffix("'") {
        token = stripped;
    }

    Some((Token::Value(token), pos))
}

fn lex_with_pattern<'text>(
    text: &'text str,
    pos: usize,
    pat: &Regex,
) -> Option<(&'text str, usize)> {
    if let Some(slice) = text.get(pos..text.len()) {
        if let Some(m) = pat.find(slice) {
            assert!(
                m.start() == 0,
                "put caret ^ to match the text from the `pos` (text is sliced to start from pos)"
            );
            return Some((m.as_str(), pos + m.end()));
        }
    }

    None
}

fn lex_symbol(text: &str, pos: usize, symbol: &'static str) -> Option<(Token<'static>, usize)> {
    if let Some(substr) = text.get(pos..) {
        if substr.starts_with(symbol) {
            return Some((Token::Symbol(symbol), pos + symbol.len()));
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_all() {
        let src = r#"
        set del delete show reveal copy history rename import secret sensitive
        all prev and or contains matches like is

        setter revealed

        name user pass url
        (=)'🦀🦀🦀''كلمة عربية مخيفة''N''' look_mom   no_spaces   'oh wow spaces'
        (zahash)('zahash')
        "#;

        use Token::*;

        match lex(src) {
            Ok(tokens) => assert_eq!(
                tokens,
                vec![
                    Keyword("set"),
                    Keyword("del"),
                    Keyword("delete"),
                    Keyword("show"),
                    Keyword("reveal"),
                    Keyword("copy"),
                    Keyword("history"),
                    Keyword("rename"),
                    Keyword("import"),
                    Keyword("secret"),
                    Keyword("sensitive"),
                    Keyword("all"),
                    Keyword("prev"),
                    Keyword("and"),
                    Keyword("or"),
                    Keyword("contains"),
                    Keyword("matches"),
                    Keyword("like"),
                    Keyword("is"),
                    Value("setter"),
                    Value("revealed"),
                    Value("name"),
                    Value("user"),
                    Value("pass"),
                    Value("url"),
                    Symbol("("),
                    Symbol("="),
                    Symbol(")"),
                    Value("🦀🦀🦀"),
                    Value("كلمة عربية مخيفة"),
                    Value("N"),
                    Value(""),
                    Value("look_mom"),
                    Value("no_spaces"),
                    Value("oh wow spaces"),
                    Symbol("("),
                    Value("zahash"),
                    Symbol(")"),
                    Symbol("("),
                    Value("zahash"),
                    Symbol(")"),
                ]
            ),

            Err(LexError::InvalidToken { pos }) => assert!(false, "{}", &src[pos..]),
        }
    }
}
