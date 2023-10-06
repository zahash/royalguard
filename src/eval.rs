use std::{collections::HashMap, fmt::Display};

use uuid::Uuid;

use crate::lex::*;
use crate::parse::*;

#[derive(Debug)]
pub enum EvaluatorError<'text> {
    LexError(LexError),
    ParseError(ParseError<'text>),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub id: Uuid,
    pub name: String,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub url: Option<String>,
}

pub struct State {
    data: HashMap<String, Data>,
}

impl<'text> State {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, query: Query<'text>) -> Vec<Data> {
        match query {
            Query::All => self.data.values().cloned().collect(),
            Query::Name(name) => self.data.get(name).into_iter().cloned().collect(),
            Query::Or(cond) => self
                .data
                .values()
                .filter(|data| cond.test(data))
                .cloned()
                .collect(),
        }
    }

    pub fn set(&mut self, name: &'text str, assignments: Vec<Assign<'text>>) {
        let data = self.data.entry(name.to_string()).or_insert(Data {
            id: Uuid::new_v4(),
            name: name.to_string(),
            user: None,
            pass: None,
            url: None,
        });

        for Assign { attr, value } in assignments {
            let value = value.to_string();
            match attr {
                "user" => data.user = Some(value),
                "pass" => data.pass = Some(value),
                "url" => data.url = Some(value),
                _ => {}
            }
        }
    }

    pub fn del(&mut self, name: &str) -> Option<Data> {
        self.data.remove(name)
    }
}

pub fn eval<'text>(
    text: &'text str,
    state: &mut State,
) -> Result<Vec<Data>, EvaluatorError<'text>> {
    let tokens = lex(text)?;
    let cmd = parse(&tokens)?;

    match cmd {
        Cmd::Set { name, assignments } => {
            state.set(name, assignments);
            Ok(vec![])
        }
        Cmd::Del { name } => Ok(state.del(name).into_iter().collect()),
        Cmd::Show(query) => Ok(state.get(query)),
        Cmd::History { name: _ } => unimplemented!("history feature coming soon"),
    }
}

pub trait Cond<'text> {
    fn test(&self, data: &Data) -> bool;
}

impl<'text> Cond<'text> for Query<'text> {
    fn test(&self, data: &Data) -> bool {
        match self {
            Query::Or(cond) => cond.test(data),
            Query::Name(name) => data.name == *name,
            Query::All => true,
        }
    }
}

impl<'text> Cond<'text> for Or<'text> {
    fn test(&self, data: &Data) -> bool {
        match self {
            Or::And(cond) => cond.test(data),
            Or::Or(lhs, rhs) => lhs.test(data) || rhs.test(data),
        }
    }
}

impl<'text> Cond<'text> for And<'text> {
    fn test(&self, data: &Data) -> bool {
        match self {
            And::Filter(cond) => cond.test(data),
            And::And(lhs, rhs) => lhs.test(data) && rhs.test(data),
        }
    }
}

impl<'text> Cond<'text> for Filter<'text> {
    fn test(&self, data: &Data) -> bool {
        match self {
            Filter::Contains(cond) => cond.test(data),
            Filter::Matches(cond) => cond.test(data),
            Filter::Cmp(cond) => cond.test(data),
            Filter::Parens(q) => q.test(data),
        }
    }
}

impl<'text> Cond<'text> for Contains<'text> {
    fn test(&self, data: &Data) -> bool {
        match self.attr {
            "name" => data.name.contains(self.substr),
            "user" => data
                .user
                .as_ref()
                .map_or(false, |user| user.contains(self.substr)),
            "pass" => data
                .pass
                .as_ref()
                .map_or(false, |pass| pass.contains(self.substr)),
            "url" => data
                .url
                .as_ref()
                .map_or(false, |url| url.contains(self.substr)),
            _ => false,
        }
    }
}

impl<'text> Cond<'text> for Matches<'text> {
    fn test(&self, data: &Data) -> bool {
        match self.attr {
            "name" => self.pat.find(&data.name).is_some(),
            "user" => data
                .user
                .as_ref()
                .map_or(false, |user| self.pat.find(&user).is_some()),
            "pass" => data
                .pass
                .as_ref()
                .map_or(false, |pass| self.pat.find(&pass).is_some()),
            "url" => data
                .url
                .as_ref()
                .map_or(false, |url| self.pat.find(&url).is_some()),
            _ => false,
        }
    }
}

impl<'text> Cond<'text> for Is<'text> {
    fn test(&self, data: &Data) -> bool {
        match self.attr {
            "name" => data.name == self.value,
            "user" => data.user.as_ref().map_or(false, |user| user == self.value),
            "pass" => data.pass.as_ref().map_or(false, |pass| pass == self.value),
            "url" => data.url.as_ref().map_or(false, |url| url == self.value),
            _ => false,
        }
    }
}

impl<'text> Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.name)?;
        if let Some(user) = &self.user {
            write!(f, "  user='{}'", user)?;
        }
        if let Some(pass) = &self.pass {
            write!(f, "  pass='{}'", pass)?;
        }
        if let Some(url) = &self.url {
            write!(f, "  url='{}'", url)?;
        }
        Ok(())
    }
}

impl<'text> From<LexError> for EvaluatorError<'text> {
    fn from(value: LexError) -> Self {
        EvaluatorError::LexError(value)
    }
}

impl<'text> From<ParseError<'text>> for EvaluatorError<'text> {
    fn from(value: ParseError<'text>) -> Self {
        EvaluatorError::ParseError(value)
    }
}
