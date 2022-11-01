#[derive(Debug)]
pub enum CANConstructionError {
    SignalSpecifiedMultipleTimes(String),
    MessageNameAlreadyExists(String),
    MessageIdAlreadyExists(u32),
}

impl std::fmt::Display for CANConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SignalSpecifiedMultipleTimes(s) => {
                write!(f, "Signal with name {s} specified multiple times.")
            }
            Self::MessageNameAlreadyExists(n) => {
                write!(f, "Message with name `{n}` already exists in network.")
            }
            Self::MessageIdAlreadyExists(i) => {
                write!(f, "Message with id 0x{:x} already exists in network.", i)
            }
        }
    }
}