use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub id: Uuid,
    pub name: String,
    pub fields: HashMap<String, String>,
}
