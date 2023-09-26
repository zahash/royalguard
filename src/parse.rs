use std::collections::HashSet;

use regex::Regex;

use crate::Token;

// <cmd> ::= add <value> {<assign>}*
//         | set <value> {<assign>}*
//         | del <value>
//         | show <query>
//         | history <value>

// <assign> ::= <attr> = <value>
// <attr> ::= user | pass | url

// <query> ::= <or> | all
// <or> ::= <and> | <or> or <and>
// <and> ::= <filter> | <and> and <filter>
// <filter> ::= <contains> | <matches> | <cmp> | <value>
// <contains> ::= <attr> contains <value>
// <matches> ::= <attr> matches <value>
// <cmp> ::= <attr> is <value>

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
            Token::Symbol("||") => {
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
    Name(&'text str),
}

fn parse_filter<'text>(
    tokens: &[Token<'text>],
    pos: usize,
) -> Result<(Filter<'text>, usize), ParseError<'text>> {
    fn parse_name<'text>(
        tokens: &[Token<'text>],
        pos: usize,
    ) -> Result<(Filter<'text>, usize), ParseError<'text>> {
        match tokens.get(pos) {
            Some(Token::Value(name)) => Ok((Filter::Name(name), pos + 1)),
            _ => Err(ParseError::ExpectedValue(pos)),
        }
    }

    combine_parsers(
        tokens,
        pos,
        &[
            Box::new(parse_contains),
            Box::new(parse_matches),
            Box::new(parse_is),
            Box::new(parse_name),
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
