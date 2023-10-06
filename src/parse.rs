use std::{collections::HashSet, fmt::Display};

use regex::Regex;

use crate::lex::*;

// <cmd> ::= add <value> {<assign>}*
//         | set <value> {<assign>}*
//         | del <value>
//         | show <query>
//         | history <value>

// <assign> ::= <attr> = <value>
// <attr> ::= user | pass | url

// <query> ::= <or> | <value> | all
// <or> ::= <and> | <or> or <and>
// <and> ::= <filter> | <and> and <filter>
// <filter> ::= ( <query> ) | <contains> | <matches> | <is>
// <contains> ::= <attr> contains <value>
// <matches> ::= <attr> matches <value>
// <is> ::= <attr> is <value>

// add 'some name with spaces' user=zahash pass=asdf url='https://asdf.com'
// set 'some name with spaces' user=zahash.z
// del 'some name'
// show name is 'some name with spaces' or (name contains asdf and url matches '.+asdf.+')
// show 'some name'
// show all

// history 'some name'

#[derive(Debug)]
pub enum ParseError<'text> {
    SyntaxError(usize, &'static str),
    ExpectedAttr(usize),
    ExpectedValue(usize),
    Expected(Token<'static>, usize),
    InvalidRegex(usize),
    DuplicateAssignments(&'text str, usize),
}

pub enum Cmd<'text> {
    Set {
        name: &'text str,
        assignments: Vec<Assign<'text>>,
    },
    Del {
        name: &'text str,
    },
    Show(Query<'text>),
    History {
        name: &'text str,
    },
}

pub fn parse<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    parse_cmd(tokens, pos)
}

fn parse_cmd<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    combine_parsers(
        tokens,
        pos,
        &[
            Box::new(parse_cmd_set),
            Box::new(parse_cmd_del),
            Box::new(parse_cmd_show),
            Box::new(parse_cmd_history),
        ],
        "cannot parse cmd",
    )
}

fn parse_cmd_set<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    let Some(Token::Keyword("set")) = tokens.get(pos) else {
        return Err(ParseError::Expected(Token::Keyword("set"), pos));
    };

    let Some(Token::Value(name)) = tokens.get(pos + 1) else {
        return Err(ParseError::ExpectedValue(pos));
    };

    let (assignments, pos) = many(tokens, pos + 2, parse_assign);

    if let Some(attr) = check_duplicate_assignments(&assignments) {
        return Err(ParseError::DuplicateAssignments(attr, pos));
    }

    Ok((Cmd::Set { name, assignments }, pos))
}

fn check_duplicate_assignments<'text>(assignments: &[Assign<'text>]) -> Option<&'text str> {
    let mut seen = HashSet::new();

    for Assign { attr, value: _ } in assignments {
        if seen.contains(attr) {
            return Some(attr);
        }
        seen.insert(attr);
    }

    None
}

fn parse_cmd_del<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    let Some(Token::Keyword("del")) = tokens.get(pos) else {
        return Err(ParseError::Expected(Token::Keyword("del"), pos));
    };

    let Some(Token::Value(name)) = tokens.get(pos + 1) else {
        return Err(ParseError::ExpectedValue(pos + 1));
    };

    Ok((Cmd::Del { name }, pos + 2))
}

fn parse_cmd_show<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    let Some(Token::Keyword("show")) = tokens.get(pos) else {
        return Err(ParseError::Expected(Token::Keyword("show"), pos));
    };

    let (query, pos) = parse_query(tokens, pos + 1)?;

    Ok((Cmd::Show(query), pos))
}

fn parse_cmd_history<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Cmd<'text>, usize), ParseError<'text>> {
    let Some(Token::Keyword("history")) = tokens.get(pos) else {
        return Err(ParseError::Expected(Token::Keyword("history"), pos));
    };

    let Some(Token::Value(name)) = tokens.get(pos + 1) else {
        return Err(ParseError::ExpectedValue(pos + 1));
    };

    Ok((Cmd::History { name }, pos + 2))
}

pub struct Assign<'text> {
    pub attr: &'text str,
    pub value: &'text str,
}

fn parse_assign<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Assign<'text>, usize), ParseError<'text>> {
    let Some(Token::Attr(attr)) = tokens.get(pos) else {
        return Err(ParseError::ExpectedAttr(pos));
    };

    let Some(Token::Symbol("=")) = tokens.get(pos + 1) else {
        return Err(ParseError::Expected(Token::Symbol("="), pos + 1));
    };

    let Some(Token::Value(value)) = tokens.get(pos + 2) else {
        return Err(ParseError::ExpectedValue(pos + 2));
    };

    Ok((Assign { attr, value }, pos + 3))
}

pub enum Query<'text> {
    Or(Or<'text>),
    All,
}

fn parse_query<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Query<'text>, usize), ParseError<'text>> {
    match tokens.get(pos) {
        Some(Token::Keyword("all")) => Ok((Query::All, pos + 1)),
        _ => {
            let (or, pos) = parse_or(tokens, pos)?;
            Ok((Query::Or(or), pos))
        }
    }
}

pub enum Or<'text> {
    And(And<'text>),
    Or(Box<Or<'text>>, And<'text>),
}

fn parse_or<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Or<'text>, usize), ParseError<'text>> {
    let (lhs, mut pos) = parse_and(tokens, pos)?;
    let mut lhs = lhs.into();
    while let Some(token) = tokens.get(pos) {
        match token {
            Token::Symbol("or") => {
                let (rhs, next_pos) = parse_and(tokens, pos + 1)?;
                pos = next_pos;
                lhs = Or::Or(Box::new(lhs), rhs);
            }
            _ => break,
        }
    }
    Ok((lhs, pos))
}

pub enum And<'text> {
    Filter(Filter<'text>),
    And(Box<And<'text>>, Filter<'text>),
}

fn parse_and<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(And<'text>, usize), ParseError<'text>> {
    let (lhs, mut pos) = parse_filter(tokens, pos)?;
    let mut lhs = lhs.into();
    while let Some(token) = tokens.get(pos) {
        match token {
            Token::Symbol("and") => {
                let (rhs, next_pos) = parse_filter(tokens, pos + 1)?;
                pos = next_pos;
                lhs = And::And(Box::new(lhs), rhs);
            }
            _ => break,
        }
    }
    Ok((lhs, pos))
}

pub enum Filter<'text> {
    Contains(Contains<'text>),
    Matches(Matches<'text>),
    Cmp(Is<'text>),
    Parens(Box<Query<'text>>),
}

fn parse_filter<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Filter<'text>, usize), ParseError<'text>> {
    fn parse_parens<'text>(
        tokens: &[Token<'text>],
        pos: usize,
    ) -> Result<(Filter<'text>, usize), ParseError<'text>> {
        let Some(Token::Symbol("(")) = tokens.get(pos) else {
            return Err(ParseError::Expected(Token::Symbol("("), pos));
        };
        let (query, pos) = parse_query(tokens, pos + 1)?;
        let Some(Token::Symbol(")")) = tokens.get(pos) else {
            return Err(ParseError::Expected(Token::Symbol(")"), pos));
        };
        Ok((Filter::Parens(Box::new(query)), pos + 1))
    }

    combine_parsers(
        tokens,
        pos,
        &[
            Box::new(parse_parens),
            Box::new(parse_contains),
            Box::new(parse_matches),
            Box::new(parse_is),
        ],
        "cannot parse filter",
    )
}

pub struct Contains<'text> {
    pub attr: &'text str,
    pub substr: &'text str,
}

fn parse_contains<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Contains<'text>, usize), ParseError<'text>> {
    let Some(Token::Attr(attr)) = tokens.get(pos) else {
        return Err(ParseError::ExpectedAttr(pos));
    };

    let Some(Token::Keyword("contains")) = tokens.get(pos + 1) else {
        return Err(ParseError::Expected(Token::Keyword("contains"), pos + 1));
    };

    let Some(Token::Value(substr)) = tokens.get(pos + 2) else {
        return Err(ParseError::ExpectedValue(pos + 2));
    };

    Ok((Contains { attr, substr }, pos + 3))
}

pub struct Matches<'text> {
    pub attr: &'text str,
    pub pat: Regex,
}

fn parse_matches<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Matches<'text>, usize), ParseError<'text>> {
    let Some(Token::Attr(attr)) = tokens.get(pos) else {
        return Err(ParseError::ExpectedAttr(pos));
    };

    let Some(Token::Keyword("matches")) = tokens.get(pos + 1) else {
        return Err(ParseError::Expected(Token::Keyword("matches"), pos + 1));
    };

    let Some(Token::Value(pat)) = tokens.get(pos + 2) else {
        return Err(ParseError::ExpectedValue(pos + 2));
    };

    let pat = Regex::new(pat).map_err(|_| ParseError::InvalidRegex(pos + 2))?;

    Ok((Matches { attr, pat }, pos + 3))
}

pub struct Is<'text> {
    pub attr: &'text str,
    pub value: &'text str,
}

fn parse_is<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Is<'text>, usize), ParseError<'text>> {
    let Some(Token::Attr(attr)) = tokens.get(pos) else {
        return Err(ParseError::ExpectedAttr(pos));
    };

    let Some(Token::Keyword("is")) = tokens.get(pos + 1) else {
        return Err(ParseError::Expected(Token::Keyword("is"), pos + 1));
    };

    let Some(Token::Value(value)) = tokens.get(pos + 2) else {
        return Err(ParseError::ExpectedValue(pos + 2));
    };

    Ok((Is { attr, value }, pos + 3))
}

fn many<'text, Ast>(
    tokens: &[Token<'text>],
    mut pos: usize,
    parser: impl Fn(&[Token<'text>], usize) -> Result<(Ast, usize), ParseError<'text>>,
) -> (Vec<Ast>, usize) {
    let mut list = vec![];

    while let Ok((ast, next_pos)) = parser(tokens, pos) {
        list.push(ast);
        pos = next_pos;
    }

    (list, pos)
}

trait Parser<'text, Ast> {
    fn parse(&self, tokens: &[Token<'text>], pos: usize)
        -> Result<(Ast, usize), ParseError<'text>>;
}

fn combine_parsers<'text, Ast>(
    tokens: &[Token<'text>],
    pos: usize,
    parsers: &[Box<dyn Parser<'text, Ast>>],
    msg: &'static str,
) -> Result<(Ast, usize), ParseError<'text>> {
    for parser in parsers {
        match parser.parse(tokens, pos) {
            Ok((ast, pos)) => return Ok((ast, pos)),
            Err(_) => continue,
        };
    }

    Err(ParseError::SyntaxError(pos, msg))
}

impl<'text, ParsedValue, F, Ast> Parser<'text, Ast> for F
where
    ParsedValue: Into<Ast>,
    F: Fn(&[Token<'text>], usize) -> Result<(ParsedValue, usize), ParseError<'text>>,
{
    fn parse(
        &self,
        tokens: &[Token<'text>],
        pos: usize,
    ) -> Result<(Ast, usize), ParseError<'text>> {
        match self(tokens, pos) {
            Ok((val, pos)) => Ok((val.into(), pos)),
            Err(e) => Err(e),
        }
    }
}

impl<'text> Display for Cmd<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cmd::Set { name, assignments } => {
                write!(f, "set '{}' ", name)?;
                write_arr(f, assignments, " ")
            }
            Cmd::Del { name } => write!(f, "del '{}'", name),
            Cmd::Show(q) => write!(f, "show {}", q),
            Cmd::History { name } => write!(f, "history {}", name),
        }
    }
}

impl<'text> Display for Assign<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.attr, self.value)
    }
}

impl<'text> Display for Query<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Or(o) => write!(f, "{}", o),
            Query::All => write!(f, "all"),
        }
    }
}

impl<'text> Display for Or<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Or::And(a) => write!(f, "{}", a),
            Or::Or(lhs, rhs) => write!(f, "({} or {})", lhs, rhs),
        }
    }
}

impl<'text> Display for And<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            And::Filter(x) => write!(f, "{}", x),
            And::And(lhs, rhs) => write!(f, "({} and {})", lhs, rhs),
        }
    }
}

impl<'text> Display for Filter<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Contains(c) => write!(f, "{}", c),
            Filter::Matches(m) => write!(f, "{}", m),
            Filter::Cmp(c) => write!(f, "{}", c),
            Filter::Parens(q) => write!(f, "({})", q),
        }
    }
}

impl<'text> Display for Contains<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} contains '{}'", self.attr, self.substr)
    }
}

impl<'text> Display for Matches<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} matches '{}'", self.attr, self.pat)
    }
}

impl<'text> Display for Is<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is '{}'", self.attr, self.value)
    }
}

fn write_arr<T>(f: &mut std::fmt::Formatter<'_>, arr: &[T], sep: &str) -> std::fmt::Result
where
    T: Display,
{
    if let Some(item) = arr.get(0) {
        write!(f, "{}", item)?;
        for item in &arr[1..] {
            write!(f, "{}{}", sep, item)?;
        }
    }

    Ok(())
}

impl<'text> From<And<'text>> for Or<'text> {
    fn from(value: And<'text>) -> Self {
        Or::And(value)
    }
}

impl<'text> From<Filter<'text>> for And<'text> {
    fn from(value: Filter<'text>) -> Self {
        And::Filter(value)
    }
}

impl<'text> From<Contains<'text>> for Filter<'text> {
    fn from(value: Contains<'text>) -> Self {
        Filter::Contains(value)
    }
}

impl<'text> From<Matches<'text>> for Filter<'text> {
    fn from(value: Matches<'text>) -> Self {
        Filter::Matches(value)
    }
}

impl<'text> From<Is<'text>> for Filter<'text> {
    fn from(value: Is<'text>) -> Self {
        Filter::Cmp(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! check {
        ($f:ident, $src:expr, $expected:expr) => {
            let tokens = lex($src).expect("** LEX ERROR");
            let (stmt, pos) = $f(&tokens, 0).expect("** Unable to parse statement");
            assert_eq!(pos, tokens.len(), "** Unable to parse all Tokens\n{}", stmt);
            let stmt = format!("{}", stmt);
            assert_eq!($expected, stmt);
        };
        ($f:ident, $src:expr) => {
            check!($f, $src, $src)
        };
    }

    macro_rules! check_ast {
        ($f:ident, $src:expr, $expected:expr) => {
            let tokens = lex($src).expect("** LEX ERROR");
            let (stmt, pos) = $f(&tokens, 0).expect("** Unable to parse statement");
            assert_eq!(pos, tokens.len());
            assert_eq!($expected, stmt);
        };
    }

    macro_rules! ast {
        ($f:ident, $src:expr) => {{
            let tokens = lex($src).expect("** LEX ERROR");
            let (stmt, pos) = $f(&tokens, 0).expect("** Unable to parse statement");
            assert_eq!(pos, tokens.len());
            stmt
        }};
    }

    #[test]
    fn test_filter() {
        // check!(parse_filter, "url contains 'github'");
        // check!(parse_filter, "user matches '[A-Z]+'");
        // check!(parse_filter, "user is 'zahash'");

        check!(
            parse_filter,
            "(url contains 'github' and user matches '[A-Z]+')"
        );
    }

    #[test]
    fn test_contains() {
        check!(parse_contains, "url contains 'github'");
    }

    #[test]
    fn test_matches() {
        check!(parse_matches, "user matches '[A-Z]+'");
    }

    #[test]
    fn test_is() {
        check!(parse_is, "user is 'zahash'");
    }
}
