use std::{collections::HashMap, fmt::Display};

use uuid::Uuid;

use crate::lex::*;
use crate::parse::*;
use crate::store::Data;

#[derive(Debug)]
pub enum EvaluatorError<'text> {
    LexError(LexError),
    ParseError(ParseError<'text>),
    ImportError(ImportError),
}

#[derive(Debug)]
pub enum ImportError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

#[derive(Clone)]
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

    pub fn import(&mut self, import: HashMap<String, HashMap<String, String>>) -> Vec<Data> {
        fn modified_name(name: &str, data: &HashMap<String, Data>) -> String {
            for offset in 1usize.. {
                let modified_name = name.to_string() + &format!("{}", offset);

                match data.contains_key(&modified_name) {
                    true => continue,
                    false => return modified_name,
                };
            }

            unreachable!(
                "reachable only when name + offset is present in data for all offset in (1usize..). which is highly unlikely."
            )
        }

        let mut imported_data = vec![];

        for (name, fields) in import {
            let name = match self.data.contains_key(&name) {
                true => modified_name(&name, &self.data),
                false => name,
            };

            let mut data = Data {
                id: Uuid::new_v4(),
                name: name.clone(),
                fields: HashMap::new(),
            };

            for (attr, value) in fields {
                data.fields.insert(attr, value);
            }

            self.data.insert(name, data.clone());
            imported_data.push(data);
        }

        imported_data
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
        Cmd::Import(fpath) => {
            let contents = std::fs::read_to_string(fpath)?;
            let data = serde_json::from_str(&contents)?;
            Ok(state.import(data))
        }
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
            "$name" | "." => data.name.contains(self.substr),
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
            "$name" | "." => self.pat.find(&data.name).is_some(),
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
            "$name" | "." => data.name == self.value,
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

impl<'text> From<std::io::Error> for EvaluatorError<'text> {
    fn from(value: std::io::Error) -> Self {
        EvaluatorError::ImportError(ImportError::IoError(value))
    }
}

impl<'text> From<serde_json::Error> for EvaluatorError<'text> {
    fn from(value: serde_json::Error) -> Self {
        EvaluatorError::ImportError(ImportError::SerdeError(value))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! check {
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
        check!(&mut state, "show all", ["'gmail'"]);

        eval!(&mut state, "set gmail user = zahash pass = supersecretpass");
        check!(
            &mut state,
            "show all",
            ["'gmail' pass='supersecretpass' user='zahash'"]
        );

        eval!(&mut state, "set gmail url = mail.google.com");
        check!(
            &mut state,
            "show all",
            ["'gmail' pass='supersecretpass' url='mail.google.com' user='zahash'"]
        );

        eval!(&mut state, "set discord url = discord.com tags = chat,call");
        check!(
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

        check!(&mut state, "delete gmail", [] as [String; 0]);

        eval!(&mut state, "set gmail url = mail.google.com");

        check!(&mut state, "delete discord", [] as [String; 0]);

        eval!(&mut state, "set discord url = discord.com");

        check!(
            &mut state,
            "delete gmail",
            ["'gmail' url='mail.google.com'"]
        );

        check!(&mut state, "show all", ["'discord' url='discord.com'"]);
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

        check!(
            &mut state,
            "show discord",
            ["'discord' pass='dpass123' url='discord.com' user='hazash'"]
        );

        check!(
            &mut state,
            "show all",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        check!(
            &mut state,
            r#"show user contains ash and url matches '\.com'"#,
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'"
            ]
        );

        check!(
            &mut state,
            r#"show url contains google or user is amogus"#,
            [
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        check!(
            &mut state,
            "show pass matches '[a-z]+123' and ( user is amogus or user contains 'ash' )",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        eval!(&mut state, "set sus user = sussolini name = potatus");
        check!(&mut state, "show name is sus", [] as [String; 0]);
        check!(
            &mut state,
            "show $name is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut state,
            "show . is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut state,
            "show name is potatus",
            ["'sus' name='potatus' user='sussolini'"]
        );
    }

    #[test]
    fn test_import() {
        let mut state = State::new();

        let mut file = tempfile::NamedTempFile::new().unwrap();
        let mut file2 = tempfile::NamedTempFile::new().unwrap();
        write!(
            file,
            "{}",
            r#"
            {
                "gmail": {
                    "user": "benito sussolini",
                    "pass": "potatus",
                    "url": "mail.google.com"
                },
                "discord": {
                    "user": "pablo susscobar",
                    "pass": "cocainum",
                    "url": "discord.com"
                }
            }
            "#
        )
        .unwrap();
        write!(file2, "{}", r#" { "gmail": {}, "discord": {} } "#).unwrap();

        let cmd = format!("import {}", file.path().to_str().unwrap());
        let cmd2 = format!("import {}", file2.path().to_str().unwrap());

        check!(
            &mut state,
            &cmd,
            [
                "'discord' pass='cocainum' url='discord.com' user='pablo susscobar'",
                "'gmail' pass='potatus' url='mail.google.com' user='benito sussolini'",
            ]
        );
        check!(
            &mut state,
            "show all",
            [
                "'discord' pass='cocainum' url='discord.com' user='pablo susscobar'",
                "'gmail' pass='potatus' url='mail.google.com' user='benito sussolini'",
            ]
        );

        check!(
            &mut state,
            &cmd,
            [
                "'discord1' pass='cocainum' url='discord.com' user='pablo susscobar'",
                "'gmail1' pass='potatus' url='mail.google.com' user='benito sussolini'",
            ]
        );
        check!(&mut state, &cmd2, ["'discord2'", "'gmail2'",]);
        check!(
            &mut state,
            "show all",
            [
                "'discord' pass='cocainum' url='discord.com' user='pablo susscobar'",
                "'discord1' pass='cocainum' url='discord.com' user='pablo susscobar'",
                "'discord2'",
                "'gmail' pass='potatus' url='mail.google.com' user='benito sussolini'",
                "'gmail1' pass='potatus' url='mail.google.com' user='benito sussolini'",
                "'gmail2'",
            ]
        );
    }
}
