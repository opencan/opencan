use std::fmt::Display;

use derive_builder::UninitializedFieldError;
use thiserror::Error;

fn maybe_space_name<T: Display>(opt: &Option<T>) -> String {
    match opt {
        Some(s) => format!(" `{s}`"),
        None => "".into(),
    }
}

#[derive(Debug, Error)]
pub enum CANConstructionError {
    #[error("Signal with name `{0}` already exists in this message.")]
    SignalNameAlreadyExists(String),

    /// Signals cannot have zero width.
    #[error("Signal with name `{0}` cannot have zero width")]
    SignalHasZeroWidth(String),

    #[error("Unable to infer width of signal{}", maybe_space_name(.0))]
    SignalWidthInferenceFailed(Option<String>),

    #[error("Refusing to infer width when width already specified of signal{}", maybe_space_name(.0))]
    SignalWidthAlreadySpecified(Option<String>),

    #[error("Enumerated value name `{0}` already exists for signal (previous value = {1});")]
    EnumeratedValueNameAlreadyExists(String, u64),

    #[error("Enumerated value `{1}` already named as `{0}`; values can only be named once")]
    EnumeratedValueValueAlreadyNamed(String, u64),

    #[error("Message with name `{0}` already exists in network.")]
    MessageNameAlreadyExists(String),

    #[error("Message with id 0x{0:x} already exists in network.")]
    MessageIdAlreadyExists(u32),

    #[error("Message name `{0}` includes invalid character `{1}`.")]
    MessageNameInvalidChar(String, char),

    #[error("Message name is empty.")]
    MessageNameEmpty,

    #[error("Node with name `{0}` already exists in network.")]
    NodeAlreadyExists(String),

    #[error("Node with name `{0}` does not exist in network.")]
    NodeDoesNotExist(String),

    #[error(
        "Signal `{0}` has start bit {1}, which precedes previous signal `{2}`'s start bit of \
            {3}. Signals must be added to message in order."
    )]
    MessageSignalsOutOfOrder(String, u32, String, u32),

    #[error("Signals `{0}` and `{1}` overlap at bit {2}.")]
    SignalsOverlap(String, String, u32),

    #[error("Signal `{0}` does not fit in message and would end at bit {1}; max is {2}")]
    SignalWillNotFitInMessage(String, u32, u32),

    #[error("Missing required field `{0}`")]
    UninitializedFieldError(String),
}

// For getting CANConstructionError from builder .build() methods
impl From<UninitializedFieldError> for CANConstructionError {
    fn from(uf: UninitializedFieldError) -> Self {
        Self::UninitializedFieldError(uf.field_name().into())
    }
}
