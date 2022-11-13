use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::error::*;

#[derive(TypedBuilder)]
pub struct CANSignalUnchecked {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignal {
    pub name: String,
    pub start_bit: u32,
    pub width: u32,

    pub description: Option<String>,
    pub offset: Option<f32>,
    pub scale: Option<f32>,
}

impl CANSignalUnchecked {
    pub fn check(self) -> Result<CANSignal, CANConstructionError> {
        if self.width == 0 {
            return Err(CANConstructionError::SignalHasZeroWidth(self.name));
        }

        Ok(CANSignal {
            name: self.name,
            start_bit: self.start_bit,
            width: self.width,
            description: self.description,
            offset: self.offset,
            scale: self.scale,
        })
    }
}
