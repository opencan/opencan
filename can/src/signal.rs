use serde::{Deserialize, Serialize};

use crate::error::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignal {
    pub offset: u32,
    pub name: String,
    pub width: u32,
    pub description: Option<String>,
}

impl CANSignal {
    pub fn new(
        offset: u32,
        name: String,
        width: u32,
        description: Option<String>,
    ) -> Result<Self, CANConstructionError> {
        if width == 0 {
            return Err(CANConstructionError::SignalHasZeroWidth(name));
        }

        Ok(CANSignal {
            offset,
            name,
            width,
            description,
        })
    }
}
