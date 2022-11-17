use std::collections::HashMap;
use std::ops::Index;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::signal::*;

const MAX_MESSAGE_BIT: u32 = 63;

#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignalWithPosition {
    // todo: make it pub(crate) and have APIs expose tuple (bit, sig)?
    pub bit: u32,
    pub sig: CANSignal,
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

    #[builder(setter(custom), field(type = "u32"))]
    pub length: u32,
}

impl CANMessageBuilder {
    /// Create a new CAN message.
    /// Message names must be at least one character long and must contain
    /// only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    pub fn build(mut self) -> Result<CANMessage, CANConstructionError> {
        // set message length in bytes to last signal's end position / 8
        self.length = self.signals.last().map_or(0, |s| {
            let end = s.bit + s.sig.width;
            if (end % 8) == 0 {
                end / 8
            } else {
                (end / 8) + 1
            }
        });

        let msg = self.__build()?;

        Self::check_name_validity(&msg.name)?;

        Ok(msg)
    }

    /// Add single signal to message.
    /// See [`add_signal_fixed()`][CANMessageBuilder::add_signal_fixed()] for more details.
    pub fn add_signal(self, sig: CANSignal) -> Result<Self, CANConstructionError> {
        let bit = self.signals.last().map_or(0, |s| s.bit + s.sig.width);

        self.add_signal_fixed(bit, sig)
    }

    /// Add single signal to message with signal position (start bit) specified.
    ///
    /// Checks:
    ///  - signal name does not repeat ([`SignalSpecifiedMultipleTimes`][CANConstructionError::SignalSpecifiedMultipleTimes])
    ///  - signals in message do not overlap ([`SignalsOverlap`][CANConstructionError::SignalsOverlap])
    ///  - signal does not extend past end of message (['SignalWillNotFitInMessage`][CANConstructionError::SignalWillNotFitInMessage])
    pub fn add_signal_fixed(
        mut self,
        bit: u32,
        sig: CANSignal,
    ) -> Result<Self, CANConstructionError> {
        if self.sig_map.contains_key(&sig.name) {
            return Err(CANConstructionError::SignalNameAlreadyExists(sig.name));
        }

        // Check signal ranges don't overlap
        // The signals are stored sorted by start bit in self.signals, so we
        // only need to ensure the last signal's end bit is no later than this
        // signal's start bit
        if let Some(last) = self.signals.last() {
            if bit <= (last.bit + last.sig.width - 1) {
                return Err(CANConstructionError::SignalsOverlap(
                    last.sig.name.clone(),
                    sig.name,
                    bit,
                ));
            }
        }

        // Check signal end position is not past the end of the message
        let new_end = bit + sig.width - 1;
        if new_end > MAX_MESSAGE_BIT {
            return Err(CANConstructionError::SignalWillNotFitInMessage(
                sig.name,
                new_end,
                MAX_MESSAGE_BIT,
            ));
        }

        self.sig_map.insert(sig.name.clone(), self.signals.len());
        self.signals.push(CANSignalWithPosition { bit, sig });

        Ok(self)
    }

    /// Add multiple signals to message, placing each signal's start position
    /// after the previous signal ends.
    ///
    /// Convenience wrapper for [`add_signal()`][CANMessageBuilder::add_signal].
    pub fn add_signals(mut self, sigs: Vec<CANSignal>) -> Result<Self, CANConstructionError> {
        for sig in sigs {
            self = self.add_signal(sig)?;
        }

        Ok(self)
    }

    /// Add multiple signals to message with signal positions (start bits) specified.
    ///
    /// Convenience wrapper for [`add_signal_fixed()`][CANMessageBuilder::add_signal_fixed].
    pub fn add_signals_fixed(
        mut self,
        sigs: Vec<(u32, CANSignal)>,
    ) -> Result<Self, CANConstructionError> {
        for (bit, sig) in sigs {
            self = self.add_signal_fixed(bit, sig)?;
        }

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
