use std::fmt::Display;

use derive_builder::UninitializedFieldError;
use thiserror::Error;

fn maybe_space_name<T: Display>(opt: &Option<T>) -> String {
    if let Some(s) = opt {
        format!(" `{}`", s)
    } else {
        "".into()
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

    #[error("Message with name `{0}` already exists in network.")]
    MessageNameAlreadyExists(String),

    #[error("Message with id 0x{0:x} already exists in network.")]
    MessageIdAlreadyExists(u32),

    #[error("Message name `{0}` includes invalid character `{1}`.")]
    MessageNameInvalidChar(String, char),

    #[error("Message name is empty.")]
    MessageNameEmpty,

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
