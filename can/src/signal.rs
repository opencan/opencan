use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;

#[derive(Serialize, Deserialize, Clone, Builder)]
#[builder(build_fn(name = "__build", error = "CANConstructionError", private))]
#[builder(pattern = "owned")]
pub struct CANSignal {
    #[builder(setter(into))]
    pub name: String,

    pub start_bit: u32,
    pub width: u32,

    #[builder(default)]
    pub description: Option<String>,

    #[builder(default)]
    pub offset: Option<f32>,

    #[builder(default)]
    pub scale: Option<f32>,
}

impl CANSignalBuilder {
    pub fn build(self) -> Result<CANSignal, CANConstructionError> {
        let s = self.__build()?;
        if s.width == 0 {
            return Err(CANConstructionError::SignalHasZeroWidth(s.name));
        }

        Ok(s)
    }

    /// Infer the width of this signal based on given information.
    /// TODO: Write and describe inference semantics.
    pub fn infer_width(self) -> Result<Self, CANConstructionError> {
        if self.width.is_some() {
            return Ok(self);
        }

        Err(CANConstructionError::SignalWidthInferenceFailed(self.name))
    }

    /// Infer the width of the signal, but return a SignalWidthAlreadySpecified
    /// error if the signal width was already specified.
    pub fn infer_width_strict(self) -> Result<Self, CANConstructionError> {
        if self.width.is_some() {
            return Err(CANConstructionError::SignalWidthAlreadySpecified(self.name));
        }

        self.infer_width()
    }
}

impl CANSignal {
    pub fn builder() -> CANSignalBuilder {
        CANSignalBuilder::default()
    }
}
