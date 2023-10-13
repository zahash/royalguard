use crate::lex::*;
use crate::parse::*;
use crate::store::Record;
use crate::store::Store;

#[derive(Debug)]
pub enum EvalError<'text> {
    LexError(LexError),
    ParseError(ParseError<'text>),
}

pub fn eval<'text>(text: &'text str, store: &mut Store) -> Result<Vec<Record>, EvalError<'text>> {
    let tokens = lex(text)?;
    let cmd = parse(&tokens)?;

    match cmd {
        Cmd::Set { name, assignments } => {
            store.set(name, assignments);
            Ok(vec![])
        }
        Cmd::Del { name } => Ok(Vec::from_iter(store.del(name))),
        Cmd::Show(query) => {
            let mut records = store.get(query);

            for record in &mut records {
                for field in &mut record.fields {
                    if field.sensitive {
                        field.value = String::from("*****")
                    }
                }
            }

            Ok(records)
        }
        Cmd::Reveal(query) => Ok(store.get(query)),
        Cmd::History { name: _ } => unimplemented!("history feature coming soon"),
        Cmd::Import(_) => unimplemented!("import feature coming soon"),
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
            "$name" | "." => data.name.contains(self.substr),
            attr => data
                .fields
                .iter()
                .find(|f| f.attr == attr)
                .map_or(false, |f| f.value.contains(self.substr)),
        }
    }
}

impl<'text> Cond<'text> for Matches<'text> {
    fn test(&self, data: &Record) -> bool {
        match self.attr {
            "$name" | "." => self.pat.find(&data.name).is_some(),
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
            "$name" | "." => data.name == self.value,
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
        EvalError::LexError(value)
    }
}

impl<'text> From<ParseError<'text>> for EvalError<'text> {
    fn from(value: ParseError<'text>) -> Self {
        EvalError::ParseError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! check {
        ($store:expr, $cmd:expr, $expected:expr) => {
            $expected.sort();

            let mut data = eval($cmd, &mut $store).expect(&format!("unable to eval {}", $cmd));
            data.sort_by(|d1, d2| d1.name.cmp(&d2.name));
            let data: Vec<String> = data.into_iter().map(|d| format!("{}", d)).collect();

            assert_eq!(data, $expected);
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

        eval!(&mut store, "set gmail url = mail.google.com");

        check!(&mut store, "delete discord", [] as [String; 0]);

        eval!(&mut store, "set discord url = discord.com");

        check!(
            &mut store,
            "delete gmail",
            ["'gmail' url='mail.google.com'"]
        );

        check!(&mut store, "show all", ["'discord' url='discord.com'"]);
    }

    #[test]
    fn test_show() {
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
            r#"show user contains ash and url matches '\.com'"#,
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
            "show $name is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut store,
            "show . is sus",
            ["'sus' name='potatus' user='sussolini'"]
        );
        check!(
            &mut store,
            "show name is potatus",
            ["'sus' name='potatus' user='sussolini'"]
        );
    }
}
