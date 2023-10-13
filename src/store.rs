use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    eval::Cond,
    parse::{Assign, Query},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    records: Vec<Record>,
}

impl<'text> Store {
    pub fn new() -> Self {
        Self { records: vec![] }
    }

    pub fn get(&self, query: Query<'text>) -> Vec<Record> {
        match query {
            Query::All => self.records.clone(),
            Query::Name(name) => {
                Vec::from_iter(self.records.iter().find(|r| r.name == name).cloned())
            }
            Query::Or(cond) => self
                .records
                .iter()
                .filter(|data| cond.test(data))
                .cloned()
                .collect(),
        }
    }

    pub fn set(&mut self, name: &'text str, assignments: Vec<Assign<'text>>) {
        let record = match self.records.iter_mut().find(|r| r.name == name) {
            Some(r) => r,
            None => {
                self.records.push(Record {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    fields: vec![],
                });
                self.records.last_mut().unwrap()
            }
        };

        for Assign {
            attr,
            value,
            sensitive,
        } in assignments
        {
            record.fields.retain(|f| f.attr != attr);
            record.fields.push(Field {
                attr: attr.to_string(),
                value: value.to_string(),
                sensitive,
            });
        }
    }

    pub fn del(&mut self, name: &str) -> Option<Record> {
        let record = self.records.iter().find(|r| r.name == name).cloned();
        self.records.retain(|r| r.name != name);
        record
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Uuid,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub attr: String,
    pub value: String,
    pub sensitive: bool,
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.name)?;

        let mut fields = self.fields.clone();
        fields.sort_by(|f1, f2| f1.attr.cmp(&f2.attr));

        for field in fields {
            write!(f, " {}", field)?;
        }
        Ok(())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}='{}'", self.attr, self.value)
    }
}
