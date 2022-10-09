use crate::value::*;

#[derive(Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,

    pub value_type: CANValueType,
}
