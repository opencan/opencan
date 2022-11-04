use thiserror::Error;

#[derive(Debug, Error)]
pub enum CANConstructionError {
    #[error("Signal with name `{0}` specified multiple times.")]
    SignalSpecifiedMultipleTimes(String),

    #[error("Message with name `{0}` already exists in network.")]
    MessageNameAlreadyExists(String),

    #[error("Message with id 0x{:x} already exists in network.", .0)]
    MessageIdAlreadyExists(u32),

    #[error("Message name `{0}` includes invalid character `{1}`.")]
    MessageNameInvalidChar(String, char),

    #[error("Message name is empty.")]
    MessageNameEmpty,
}
