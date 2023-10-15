use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    eval::Cond,
    parse::{Assign, Query},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    records: Vec<Record>,
    version: String,
}

pub enum RenameStatus {
    OldNameNotFound,
    NewNameAlreadyExists,
    Successful,
}

impl<'text> Store {
    pub fn new() -> Self {
        Self {
            records: vec![],
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
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
                    history: vec![],
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

        record.update_history();
    }

    pub fn rename(&mut self, old: &str, new: &str) -> RenameStatus {
        if self.records.iter().find(|r| r.name == new).is_some() {
            return RenameStatus::NewNameAlreadyExists;
        };

        let Some(record) = self.records.iter_mut().find(|r| r.name == old) else {
            return RenameStatus::OldNameNotFound;
        };

        record.name = new.into();
        RenameStatus::Successful
    }

    pub fn history(&self, name: &str) -> Vec<HistoryEntry> {
        match self.records.iter().find(|r| r.name == name) {
            Some(record) => record.history.clone(),
            None => vec![],
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<Record> {
        let record = self.records.iter().find(|r| r.name == name).cloned();
        self.records.retain(|r| r.name != name);
        record
    }

    pub fn remove_attrs(&mut self, name: &str, attrs: &[&str]) -> Option<Record> {
        if let Some(record) = self.records.iter_mut().find(|r| r.name == name) {
            record.fields.retain(|f| !attrs.contains(&f.attr.as_str()));
            record.update_history();
            return Some(record.clone());
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Uuid,
    pub name: String,
    pub fields: Vec<Field>,

    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

impl Record {
    pub fn update_history(&mut self) {
        self.history.sort_by(|h1, h2| h1.datetime.cmp(&h2.datetime));
        match self.history.last_mut() {
            Some(history) => {
                history.fields.sort_by(|f1, f2| f1.attr.cmp(&f2.attr));
                self.fields.sort_by(|f1, f2| f1.attr.cmp(&f2.attr));
                if history.fields != self.fields {
                    self.history.push(HistoryEntry::new(self.fields.clone()))
                }
            }
            None => self.history.push(HistoryEntry::new(self.fields.clone())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Field {
    pub attr: String,
    pub value: String,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub datetime: DateTime<Local>,
    pub fields: Vec<Field>,
}

impl HistoryEntry {
    pub fn new(fields: Vec<Field>) -> Self {
        Self {
            datetime: Local::now(),
            fields,
        }
    }
}
