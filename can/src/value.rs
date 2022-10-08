#[derive(Clone)]
pub struct CANValueTypeInteger {
    pub length: i32,
    pub signed: bool,
}

#[derive(Clone)]
pub enum CANValueType {
    Integer(CANValueTypeInteger),
}

impl std::fmt::Display for CANValueTypeInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (if self.signed { "s" } else { "u" }),
            self.length
        )
    }
}

impl std::fmt::Display for CANValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CANValueType::Integer(s) => s.fmt(f),
        }
    }
}
