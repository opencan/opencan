use std::collections::HashMap;
use std::ops::Index;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::signal::*;

#[derive(Serialize, Deserialize, Clone, Builder)]
#[builder(build_fn(name = "__build", error = "CANConstructionError", private))]
pub struct CANMessage {
    pub name: String,
    pub id: u32,

    #[builder(default)]
    pub cycletime_ms: Option<u32>,

    pub signals: Vec<CANSignal>,

    #[builder(setter(skip))]
    #[serde(skip)]
    pub sig_map: HashMap<String, usize>,
}

impl CANMessageBuilder {
    /// Create a new CAN message.
    /// Message names must be at least one character long and must contain
    /// only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    // todo: check that signals fit within message and do not overlap
    pub fn build(&self) -> Result<CANMessage, CANConstructionError> {
        let mut msg = self.__build()?; // fix unwrap

        let mut sig_map = HashMap::new();

        Self::check_name_validity(&msg.name)?;

        for (i, sig) in msg.signals.iter().enumerate() {
            if sig_map.contains_key(&sig.name) {
                return Err(CANConstructionError::SignalSpecifiedMultipleTimes(
                    sig.name.clone(),
                ));
            }

            sig_map.insert(sig.name.to_string(), i);
        }

        msg.sig_map = sig_map;

        Ok(msg)
    }

    fn check_name_validity(name: &str) -> Result<(), CANConstructionError> {
        if name.is_empty() {
            return Err(CANConstructionError::MessageNameEmpty);
        }

        if let Some(c) = name
            .chars()
            .find(|c| (!c.is_ascii_alphanumeric()) && c != &'_')
        {
            return Err(CANConstructionError::MessageNameInvalidChar(name.into(), c));
        }

        Ok(())
    }
}

impl CANMessage {
    pub fn builder() -> CANMessageBuilder {
        CANMessageBuilder::default()
    }

    pub fn get_sig(&self, name: &str) -> Option<&CANSignal> {
        let idx = self.sig_map.get(name)?;

        // unwrap here, as signals really should have the signal if sig_map does
        Some(self.signals.get(*idx).unwrap())
    }
}

// Easy indexing of msg["signal"]. Panics if signal absent.
// (no, this can't be an Option, the Index trait doesn't allow it)
impl Index<&str> for CANMessage {
    type Output = CANSignal;

    fn index(&self, index: &str) -> &Self::Output {
        return self.get_sig(index).unwrap();
    }
}
