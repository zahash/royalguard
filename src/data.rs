use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub attr: String,
    pub value: String,
    pub sensitive: bool,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sensitive {
            true => write!(f, "{}=*****", self.attr),
            false => write!(f, "{}='{}'", self.attr, self.value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub id: Uuid,
    pub name: String,
    pub fields: Vec<Field>,
}

impl Display for Data {
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
