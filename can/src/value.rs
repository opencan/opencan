#[derive(Clone)]
pub struct CANValueTypeInteger {
    pub length: i32,
    pub signed: bool,
}

#[derive(Clone)]
pub enum CANValueType {
    Integer(CANValueTypeInteger),
}
