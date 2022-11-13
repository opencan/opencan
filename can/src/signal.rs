use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,
    pub description: Option<String>,
}
