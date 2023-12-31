use anyhow::anyhow;
use arboard::Clipboard;
use ignorant::Ignore;

use crate::lex::*;
use crate::parse::*;
use crate::store::Field;
use crate::store::HistoryEntry;
use crate::store::Record;
use crate::store::RenameStatus;
use crate::store::Store;

#[derive(Debug)]
pub enum EvalError<'text> {
    Lex(LexError),
    Parse(ParseError<'text>),
    Import(anyhow::Error),
}

pub enum Evaluation<'text> {
    Set,
    Del(Option<Record>),
    Show(Vec<Record>),
    Reveal(Vec<Record>),
    Copy(bool),
    History(Vec<HistoryEntry>),
    RevealHistory(Vec<HistoryEntry>),
    Import(usize),
    Rename((RenameStatus, &'text str, &'text str)),
}

impl<'text> Evaluation<'text> {
    fn fmt_record(record: Record, sensitize: bool) -> String {
        use std::fmt::Write;

        let mut buf = String::new();
        write!(buf, "'{}'", record.name).ignore();
        Self::fmt_fields(record.fields, sensitize, &mut buf);

        buf
    }

    fn fmt_history(history: HistoryEntry, sensitize: bool) -> String {
        use std::fmt::Write;

        let mut buf = String::new();
        write!(buf, "({})", history.datetime.format("%Y-%m-%d %H:%M %:z")).ignore();
        Self::fmt_fields(history.fields, sensitize, &mut buf);

        buf
    }

    fn fmt_fields(mut fields: Vec<Field>, sensitize: bool, buf: &mut String) {
        use std::fmt::Write;

        fields.sort_by(|f1, f2| f1.attr.cmp(&f2.attr));

        for field in fields {
            match sensitize && field.sensitive {
                true => write!(buf, " {}=*****", field.attr),
                false => write!(buf, " {}='{}'", field.attr, field.value),
            }
            .ignore()
        }
    }

    pub fn lines(self) -> Vec<String> {
        match self {
            Evaluation::Set => vec![],
            Evaluation::Del(record) => match record {
                Some(record) => vec![Evaluation::fmt_record(record, true)],
                None => vec![],
            },
            Evaluation::Show(mut records) => {
                records.sort_by(|r1, r2| r1.name.cmp(&r2.name));
                records
                    .into_iter()
                    .map(|record| Evaluation::fmt_record(record, true))
                    .collect()
            }
            Evaluation::Reveal(mut records) => {
                records.sort_by(|r1, r2| r1.name.cmp(&r2.name));
                records
                    .into_iter()
                    .map(|record| Evaluation::fmt_record(record, false))
                    .collect()
            }
            Evaluation::Copy(status) => match status {
                true => vec!["Copied!".into()],
                false => vec!["Unable to Copy! Try Again!".into()],
            },
            Evaluation::History(mut history) => {
                history.sort_by(|h1, h2| h1.datetime.cmp(&h2.datetime).reverse());
                history
                    .into_iter()
                    .map(|h| Evaluation::fmt_history(h, true))
                    .collect()
            }
            Evaluation::RevealHistory(mut history) => {
                history.sort_by(|h1, h2| h1.datetime.cmp(&h2.datetime).reverse());
                history
                    .into_iter()
                    .map(|h| Evaluation::fmt_history(h, false))
                    .collect()
            }
            Evaluation::Rename((status, old, new)) => match status {
                RenameStatus::OldNameNotFound => vec![format!("'{}' not found!", old)],
                RenameStatus::NewNameAlreadyExists => vec![format!("'{}' already exists!", new)],
                RenameStatus::Successful => vec!["Renamed!".into()],
            },
            Evaluation::Import(nrecords) => vec![format!("imported {} records", nrecords)],
        }
    }
}

pub fn eval<'text>(
    text: &'text str,
    store: &mut Store,
) -> Result<Evaluation<'text>, EvalError<'text>> {
    let tokens = lex(text)?;
    let cmd = parse(&tokens)?;

    match cmd {
        Cmd::Set { name, assignments } => {
            store.set(name, assignments);
            Ok(Evaluation::Set)
        }
        Cmd::Del { name, attrs } => match attrs.as_slice() {
            [] => Ok(Evaluation::Del(store.remove(name))),
            attrs => Ok(Evaluation::Del(store.remove_attrs(name, attrs))),
        },
        Cmd::Show(query) => Ok(Evaluation::Show(store.get(query))),
        Cmd::Reveal(query) => Ok(Evaluation::Reveal(store.get(query))),
        Cmd::Copy { name, attr } => {
            if let Some(record) = store.get(Query::Name(name)).pop() {
                if let Some(field) = record.fields.iter().find(|f| f.attr == attr) {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        return Ok(Evaluation::Copy(
                            clipboard.set_text(field.value.clone()).is_ok(),
                        ));
                    }
                }
            }
            Ok(Evaluation::Copy(false))
        }
        Cmd::History(name) => Ok(Evaluation::History(store.history(name))),
        Cmd::RevealHistory(name) => Ok(Evaluation::RevealHistory(store.history(name))),
        Cmd::Rename(old, new) => {
            let status = store.rename(old, new);
            Ok(Evaluation::Rename((status, old, new)))
        }
        Cmd::Import(fpath) => {
            let content =
                std::fs::read_to_string(fpath).map_err(|e| EvalError::Import(anyhow!(e)))?;

            for (line_idx, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }

                let cmd = String::from("set ") + line;

                if let Err(e) = eval(&cmd, store) {
                    return Err(EvalError::Import(anyhow!(
                        "{:?} line number: [{}] {}",
                        e,
                        line_idx + 1,
                        line,
                    )));
                }
            }

            Ok(Evaluation::Import(content.lines().count()))
        }
    }
}

pub trait Cond<'text> {
    fn test(&self, data: &Record) -> bool;
}

impl<'text> Cond<'text> for Query<'text> {
    fn test(&self, data: &Record) -> bool {
        match self {
            Query::Or(cond) => cond.test(data),
            Query::Name(name) => data.name == *name,
            Query::All => true,
        }
    }
}

impl<'text> Cond<'text> for Or<'text> {
    fn test(&self, data: &Record) -> bool {
        match self {
            Or::And(cond) => cond.test(data),
            Or::Or(lhs, rhs) => lhs.test(data) || rhs.test(data),
        }
    }
}

impl<'text> Cond<'text> for And<'text> {
    fn test(&self, data: &Record) -> bool {
        match self {
            And::Filter(cond) => cond.test(data),
            And::And(lhs, rhs) => lhs.test(data) && rhs.test(data),
        }
    }
}

impl<'text> Cond<'text> for Filter<'text> {
    fn test(&self, data: &Record) -> bool {
        match self {
            Filter::Contains(cond) => cond.test(data),
            Filter::Matches(cond) => cond.test(data),
            Filter::Cmp(cond) => cond.test(data),
            Filter::Parens(q) => q.test(data),
        }
    }
}

impl<'text> Cond<'text> for Contains<'text> {
    fn test(&self, data: &Record) -> bool {
        match self.attr {
            "." => data
                .name
                .to_lowercase()
                .contains(&self.substr.to_lowercase()),
            attr => data
                .fields
                .iter()
                .find(|f| f.attr == attr)
                .map_or(false, |f| {
                    f.value.to_lowercase().contains(&self.substr.to_lowercase())
                }),
        }
    }
}

impl<'text> Cond<'text> for Matches<'text> {
    fn test(&self, data: &Record) -> bool {
        match self.attr {
            "." => self.pat.find(&data.name).is_some(),
            attr => data
                .fields
                .iter()
                .find(|f| f.attr == attr)
                .and_then(|f| self.pat.find(&f.value))
                .is_some(),
        }
    }
}

impl<'text> Cond<'text> for Is<'text> {
    fn test(&self, data: &Record) -> bool {
        match self.attr {
            "." => data.name == self.value,
            attr => data
                .fields
                .iter()
                .find(|f| f.attr == attr)
                .map_or(false, |f| f.value == self.value),
        }
    }
}

impl<'text> From<LexError> for EvalError<'text> {
    fn from(value: LexError) -> Self {
        EvalError::Lex(value)
    }
}

impl<'text> From<ParseError<'text>> for EvalError<'text> {
    fn from(value: ParseError<'text>) -> Self {
        EvalError::Parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! check {
        ($store:expr, $cmd:expr, $expected:expr) => {
            let eval = eval($cmd, &mut $store).expect(&format!("unable to eval {}", $cmd));
            assert_eq!(eval.lines(), $expected);
        };
    }

    macro_rules! eval {
        ($store:expr, $($cmd:expr),*) => {
            $ ( eval($cmd, $store).expect(&format!("unable to eval {}", $cmd)); )*
        };
    }

    #[test]
    fn test_set() {
        let mut store = Store::new();

        eval!(&mut store, "set gmail");
        check!(&mut store, "show all", ["'gmail'"]);

        eval!(&mut store, "set gmail user = zahash pass = supersecretpass");
        check!(
            &mut store,
            "show all",
            ["'gmail' pass='supersecretpass' user='zahash'"]
        );

        eval!(&mut store, "set gmail url = mail.google.com");
        check!(
            &mut store,
            "show all",
            ["'gmail' pass='supersecretpass' url='mail.google.com' user='zahash'"]
        );

        eval!(&mut store, "set gmail pass = updatedpass");
        check!(
            &mut store,
            "show all",
            ["'gmail' pass='updatedpass' url='mail.google.com' user='zahash'"]
        );

        eval!(&mut store, "set discord url = discord.com tags = chat,call");
        check!(
            &mut store,
            "show all",
            [
                "'discord' tags='chat,call' url='discord.com'",
                "'gmail' pass='updatedpass' url='mail.google.com' user='zahash'",
            ]
        );
    }

    #[test]
    fn test_del() {
        let mut store = Store::new();

        check!(&mut store, "delete gmail", [] as [String; 0]);

        eval!(
            &mut store,
            "set gmail url = mail.google.com sensitive pass = gpass"
        );

        check!(&mut store, "delete discord", [] as [String; 0]);

        eval!(
            &mut store,
            "set discord user = doubledragon url = discord.com"
        );

        check!(
            &mut store,
            "delete gmail",
            ["'gmail' pass=***** url='mail.google.com'"]
        );

        check!(
            &mut store,
            "show all",
            ["'discord' url='discord.com' user='doubledragon'"]
        );

        check!(&mut store, "delete gmail user pass", [] as [String; 0]);
        check!(
            &mut store,
            "delete discord user pass",
            ["'discord' url='discord.com'"]
        );
    }

    #[test]
    fn test_show_reveal() {
        let mut store = Store::new();

        eval!(
            &mut store,
            "set gmail user = zahash pass = pass123 url = mail.google.com",
            "set discord user = hazash pass = dpass123 url = discord.com",
            "set twitch user = amogus pass = tpass123"
        );

        check!(
            &mut store,
            "show discord",
            ["'discord' pass='dpass123' url='discord.com' user='hazash'"]
        );
        check!(
            &mut store,
            "show all",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );
        check!(
            &mut store,
            r#"show user contains AsH and url matches '\.com'"#,
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'"
            ]
        );
        check!(
            &mut store,
            r#"show url contains google or user is amogus"#,
            [
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );
        check!(
            &mut store,
            "show pass matches '[a-z]+123' and ( user is amogus or user contains 'ash' )",
            [
                "'discord' pass='dpass123' url='discord.com' user='hazash'",
                "'gmail' pass='pass123' url='mail.google.com' user='zahash'",
                "'twitch' pass='tpass123' user='amogus'"
            ]
        );

        eval!(&mut store, "set sus user = sussolini name = potatus");
        check!(&mut store, "show name is sus", [] as [String; 0]);
        check!(
            &mut store,
            "show name is potatus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut store,
            "show . is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut store,
            "show . contains SUS",
            ["'sus' name='potatus' user='sussolini'"]
        );

        eval!(&mut store, "set sus secret pass = supahotfire");
        check!(
            &mut store,
            "show sus",
            ["'sus' name='potatus' pass=***** user='sussolini'"]
        );

        eval!(&mut store, "set sus sensitive user = sussolini");
        check!(
            &mut store,
            "show sus",
            ["'sus' name='potatus' pass=***** user=*****"]
        );
        check!(
            &mut store,
            "reveal sus",
            ["'sus' name='potatus' pass='supahotfire' user='sussolini'"]
        );
    }

    #[test]
    fn test_history() {
        let mut store = Store::new();

        eval!(
            &mut store,
            "set sus user = 'benito sussolini' sensitive pass = amogus"
        );
        eval!(&mut store, "set sus user = 'pablo susscobar'");
        eval!(&mut store, "set sus user = 'pablo susscobar'");
        eval!(&mut store, "del sus user");
        eval!(&mut store, "set sus pass = potatus");
        eval!(&mut store, "set sus note = 'this is the latest'");

        check!(
            &mut store,
            "show sus",
            ["'sus' note='this is the latest' pass='potatus'"]
        );
        match eval("history sus", &mut store).unwrap().lines().as_slice() {
            [h1, h2, h3, h4, h5] => {
                assert!(h1.ends_with("note='this is the latest' pass='potatus'"));
                assert!(h2.ends_with("pass='potatus'"));
                assert!(h3.ends_with("pass=*****"));
                assert!(h4.ends_with("pass=***** user='pablo susscobar'"));
                assert!(h5.ends_with("pass=***** user='benito sussolini'"));
            }
            _ => assert!(false),
        }

        check!(&mut store, "history blah", [] as [String; 0]);
    }

    #[test]
    fn test_reveal_history() {
        let mut store = Store::new();

        eval!(
            &mut store,
            "set sus user = 'benito sussolini' sensitive pass = amogus"
        );
        match eval("reveal history sus", &mut store)
            .unwrap()
            .lines()
            .as_slice()
        {
            [h1] => assert!(h1.ends_with("pass='amogus' user='benito sussolini'")),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_rename() {
        let mut store = Store::new();

        check!(&mut store, "rename gmail discord", ["'gmail' not found!"]);

        eval!(&mut store, "set discord");
        check!(
            &mut store,
            "rename gmail discord",
            ["'discord' already exists!"]
        );

        check!(&mut store, "rename discord discord2", ["Renamed!"]);
    }

    #[test]
    fn test_copy() {
        let mut store = Store::new();

        check!(&mut store, "copy gmail pass", ["Unable to Copy! Try Again!"]);

        eval!(&mut store, "set gmail");
        check!(&mut store, "copy gmail pass", ["Unable to Copy! Try Again!"]);

        eval!(&mut store, "set gmail url = mail.google.com");
        check!(&mut store, "copy gmail pass", ["Unable to Copy! Try Again!"]);

        eval!(&mut store, "set gmail pass = gpass");
        check!(&mut store, "copy gmail pass", ["Copied!"]);

        eval!(&mut store, "set gmail sensitive pass = gpass");
        check!(&mut store, "copy gmail pass", ["Copied!"]);
    }

    #[test]
    fn test_import() {
        use std::io::Write;

        fn import(store: &mut Store, contents: &'static str) {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();
            let cmd = format!("import {}", file.path().to_str().unwrap());
            eval!(store, &cmd);
        }

        let mut store = Store::new();

        import(&mut store, "");
        check!(&mut store, "show all", [] as [String; 0]);

        import(
            &mut store,
            r#"
            'gmail' user = ligma pass = balls
            'gmail' user = ligma pass = balls
            'gmail' user = ligma pass = balls
            'gmail' user = ligma pass = balls
            'gmail' user = 'benito sussolini' pass = 'joseph ballin'
            'gmail' user = 'benito sussolini' pass = 'joseph ballin'
            'gmail' user = 'benito sussolini' pass = 'joseph ballin'
            'discord' user = 'dorito breath' pass = 'kitten'
            'discord' user = 'dorito breath' pass = 'kitten'
            "#,
        );
        check!(
            &mut store,
            "show all",
            [
                "'discord' pass='kitten' user='dorito breath'",
                "'gmail' pass='joseph ballin' user='benito sussolini'"
            ]
        );
        match eval("history gmail", &mut store)
            .unwrap()
            .lines()
            .as_slice()
        {
            [h1, h2] => {
                assert!(h1.ends_with("pass='joseph ballin' user='benito sussolini'"));
                assert!(h2.ends_with("pass='balls' user='ligma'"));
            }
            _ => assert!(false),
        }
        match eval("history discord", &mut store)
            .unwrap()
            .lines()
            .as_slice()
        {
            [h1] => assert!(h1.ends_with("pass='kitten' user='dorito breath'")),
            _ => assert!(false),
        }
    }
}
