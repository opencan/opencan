use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CANValueTypeInteger {
    pub length: i32,
    pub signed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CANValueType {
    Integer(CANValueTypeInteger),
}
