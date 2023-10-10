use std::{collections::HashMap, fmt::Display};

use uuid::Uuid;

use crate::lex::*;
use crate::parse::*;
use crate::store::Data;

#[derive(Debug)]
pub enum EvaluatorError<'text> {
    LexError(LexError),
    ParseError(ParseError<'text>),
}

pub struct State {
    data: HashMap<String, Data>,
}

impl From<Vec<Data>> for State {
    fn from(data: Vec<Data>) -> Self {
        let mut state = State::new();

        for d in data {
            state.data.insert(d.name.clone(), d);
        }

        state
    }
}

impl From<State> for Vec<Data> {
    fn from(state: State) -> Self {
        state.data.into_iter().map(|(_, v)| v).collect()
    }
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
            fields: HashMap::new(),
        });

        for Assign { attr, value } in assignments {
            data.fields.insert(attr.to_string(), value.to_string());
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
            "$name" => data.name.contains(self.substr),
            attr => data
                .fields
                .get(attr)
                .map_or(false, |val| val.contains(self.substr)),
        }
    }
}

impl<'text> Cond<'text> for Matches<'text> {
    fn test(&self, data: &Data) -> bool {
        match self.attr {
            "$name" => self.pat.find(&data.name).is_some(),
            attr => data
                .fields
                .get(attr)
                .and_then(|val| self.pat.find(val))
                .is_some(),
        }
    }
}

impl<'text> Cond<'text> for Is<'text> {
    fn test(&self, data: &Data) -> bool {
        match self.attr {
            "$name" => data.name == self.value,
            attr => data.fields.get(attr).map_or(false, |val| val == self.value),
        }
    }
}

impl<'text> Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.name)?;

        let mut fields = self.fields.iter().collect::<Vec<(&String, &String)>>();
        fields.sort_by_key(|&(k, _)| k);

        for (k, v) in fields {
            write!(f, " {}='{}'", k, v)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! check_read {
        ($state:expr, $cmd:expr, $expected:expr) => {
            $expected.sort();

            let mut data = eval($cmd, &mut $state).expect(&format!("unable to eval {}", $cmd));
            data.sort_by(|d1, d2| d1.name.cmp(&d2.name));
            let data: Vec<String> = data.into_iter().map(|d| format!("{}", d)).collect();

            assert_eq!(data, $expected);
        };
    }

    macro_rules! eval {
        ($state:expr, $($cmd:expr),*) => {
            $ ( eval($cmd, $state).expect(&format!("unable to eval {}", $cmd)); )*
        };
    }

    #[test]
    fn test_set() {
        let mut state = State::new();

        eval!(&mut state, "set gmail");
        check_read!(&mut state, "show all", ["'gmail'"]);

        eval!(&mut state, "set gmail user = zahash pass = supersecretpass");
        check_read!(
            &mut state,
            "show all",
            ["'gmail' pass='supersecretpass' user='zahash'"]
        );

        eval!(&mut state, "set gmail url = mail.google.com");
        check_read!(
            &mut state,
            "show all",
            ["'gmail' pass='supersecretpass' url='mail.google.com' user='zahash'"]
        );

        eval!(&mut state, "set discord url = discord.com tags = chat,call");
        check_read!(
            &mut state,
            "show all",
            [
                "'discord' tags='chat,call' url='discord.com'",
                "'gmail' pass='supersecretpass' url='mail.google.com' user='zahash'",
            ]
        );
    }

    #[test]
    fn test_del() {
        let mut state = State::new();

        check_read!(&mut state, "delete gmail", [] as [String; 0]);

        eval!(&mut state, "set gmail url = mail.google.com");

        check_read!(&mut state, "delete discord", [] as [String; 0]);

        eval!(&mut state, "set discord url = discord.com");

        check_read!(
            &mut state,
            "delete gmail",
            ["'gmail' url='mail.google.com'"]
        );

        check_read!(&mut state, "show all", ["'discord' url='discord.com'"]);
    }

    #[test]
    fn test_show() {
        let mut state = State::new();

        eval!(
            &mut state,
            "set gmail user = zahash pass = pass123 url = mail.google.com",
            "set discord user = hazash pass = dpass123 url = discord.com",
            "set twitch user = amogus pass = tpass123"
        );

        check_read!(
            &mut state,
            "show discord",
            ["'discord' pass='dpass123' url='discord.com' user='hazash'"]
        );

        check_read!(
            &mut state,
            "show all",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        check_read!(
            &mut state,
            r#"show user contains ash and url matches '\.com'"#,
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'"
            ]
        );

        check_read!(
            &mut state,
            r#"show url contains google or user is amogus"#,
            [
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        check_read!(
            &mut state,
            "show pass matches '[a-z]+123' and ( user is amogus or user contains 'ash' )",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        eval!(&mut state, "set sus user = sussolini name = potatus");
        check_read!(&mut state, "show name is sus", [] as [String; 0]);
        check_read!(
            &mut state,
            "show $name is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check_read!(
            &mut state,
            "show name is potatus",
            ["'sus' name='potatus' user='sussolini'"]
        );
    }
}
