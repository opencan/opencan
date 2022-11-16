use std::collections::HashMap;
use std::ops::Index;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::signal::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignalWithPosition { // todo: make it pub(crate) and have APIs expose tuple (bit, sig)?
    pub bit: u32,
    pub sig: CANSignal
}

#[derive(Serialize, Deserialize, Clone, Builder)]
#[builder(build_fn(name = "__build", error = "CANConstructionError", private))]
#[builder(pattern = "owned")]
pub struct CANMessage {
    #[builder(setter(into))]
    pub name: String,
    pub id: u32,

    #[builder(default)]
    pub cycletime_ms: Option<u32>,

    // skip builder because we will provide add_signals() instead
    #[builder(setter(custom), field(type = "Vec<CANSignalWithPosition>"))]
    pub signals: Vec<CANSignalWithPosition>,

    #[builder(setter(custom), field(type = "HashMap<String, usize>"))]
    #[serde(skip)]
    pub sig_map: HashMap<String, usize>,
}

impl CANMessageBuilder {
    /// Create a new CAN message.
    /// Message names must be at least one character long and must contain
    /// only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    // todo: check that signals fit within message and do not overlap
    pub fn build(self) -> Result<CANMessage, CANConstructionError> {
        let msg = self.__build()?;

        Self::check_name_validity(&msg.name)?;

        Ok(msg)
    }

    /// Add multiple signals to message.
    /// Convenience wrapper for add_signal.
    pub fn add_signals(mut self, sigs: Vec<CANSignal>) -> Result<Self, CANConstructionError> {
        for sig in sigs {
            self = self.add_signal(sig)?;
        }

        Ok(self)
    }

    /// Add single signal to message.
    /// Checks:
    ///  - signal name does not repeat (SignalSpecifiedMultipleTimes)
    pub fn add_signal(mut self, sig: CANSignal) -> Result<Self, CANConstructionError> {
        if self.sig_map.contains_key(&sig.name) {
            return Err(CANConstructionError::SignalSpecifiedMultipleTimes(sig.name));
        }

        self.sig_map.insert(sig.name.clone(), self.signals.len());
        self.signals.push(CANSignalWithPosition {bit: 0, sig}); // todo

        Ok(self)
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

    pub fn get_sig(&self, name: &str) -> Option<&CANSignalWithPosition> {
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
        return &self.get_sig(index).unwrap().sig;
    }
}
