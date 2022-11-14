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

    // skip builder because we will provide add_signals() instead
    #[builder(setter(custom))]
    pub signals: Vec<CANSignal>,

    #[builder(setter(custom))]
    #[serde(skip)]
    pub sig_map: HashMap<String, usize>,
}

impl CANMessageBuilder {
    /// Create a new CAN message.
    /// Message names must be at least one character long and must contain
    /// only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    // todo: check that signals fit within message and do not overlap
    pub fn build(&mut self) -> Result<CANMessage, CANConstructionError> {
        self.ensure_signals_init();

        let msg = self.__build()?;

        Self::check_name_validity(&msg.name)?;

        Ok(msg)
    }

    /// Add multiple signals to message.
    /// Convenience wrapper for add_signal.
    pub fn add_signals(&mut self, sigs: Vec<CANSignal>) -> Result<&mut Self, CANConstructionError> {
        for sig in sigs {
            self.add_signal(sig)?;
        }

        Ok(self)
    }

    /// Add single signal to message.
    /// Checks:
    ///  - signal name does not repeat (SignalSpecifiedMultipleTimes)
    pub fn add_signal(&mut self, sig: CANSignal) -> Result<&mut Self, CANConstructionError> {
        self.ensure_signals_init();

        let sig_map = self.sig_map.as_mut().unwrap();
        let signals = self.signals.as_mut().unwrap();

        if sig_map.contains_key(&sig.name) {
            return Err(CANConstructionError::SignalSpecifiedMultipleTimes(sig.name));
        }

        sig_map.insert(sig.name.clone(), signals.len());
        signals.push(sig);

        Ok(self)
    }

    fn ensure_signals_init(&mut self) {
        if self.signals.is_none() {
            self.signals = Some(vec![]);
            self.sig_map = Some(HashMap::new());
        }
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
