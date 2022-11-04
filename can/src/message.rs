use std::collections::HashMap;
use std::ops::Index;

use crate::error::*;
use crate::signal::*;

#[derive(Clone)]
pub struct CANMessageDesc {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
}

pub struct CANMessage {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
    pub sig_map: HashMap<String, usize>,
}

impl CANMessage {
    /// Create a new CAN message.
    /// Message names must be at least one character long and must contain
    /// only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    // todo: check that signals fit within message and do not overlap
    pub fn new(desc: CANMessageDesc) -> Result<Self, CANConstructionError> {
        let mut sigs = Vec::new();
        let mut sig_map = HashMap::new();

        Self::check_name_validity(&desc.name)?;

        for sig in desc.signals {
            if sig_map.contains_key(&sig.name) {
                return Err(CANConstructionError::SignalSpecifiedMultipleTimes(sig.name));
            }

            sig_map.insert(sig.name.to_string(), sigs.len());
            sigs.push(sig);
        }

        Ok(CANMessage {
            name: desc.name,
            id: desc.id,
            signals: sigs,
            sig_map,
        })
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
