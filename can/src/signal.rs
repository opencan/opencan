use serde::{Deserialize, Serialize};

use crate::value::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,
    pub description: Option<String>,

    pub value_type: CANValueType,
}
