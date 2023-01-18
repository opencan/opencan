use std::collections::HashMap;
use std::ops::Index;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::signal::*;

const MAX_MESSAGE_BIT: u32 = 63;

/// [`CANSignal`] with its position (start bit) in its message.
#[derive(Serialize, Deserialize, Clone)]
pub struct CANSignalWithPosition {
    bit: u32,
    pub sig: CANSignal,
}

impl CANSignalWithPosition {
    /// Start bit of signal.
    pub const fn start(&self) -> u32 {
        self.bit
    }

    /// Last bit of signal.
    pub const fn end(&self) -> u32 {
        self.start() + self.sig.width - 1
    }

    fn name_clone(&self) -> String {
        self.sig.name.clone()
    }
}

/// A validated description of a CAN message.
#[derive(Serialize, Deserialize, Clone, Builder)]
#[builder(build_fn(name = "__build", error = "CANConstructionError", private))]
#[builder(pattern = "owned")]
pub struct CANMessage {
    /// Message name.
    #[builder(setter(into))]
    pub name: String,

    /// Message ID.
    // todo - extended/non-extended as enum
    pub id: u32,

    /// Message cycle time in milliseconds.
    #[builder(default)]
    pub cycletime: Option<u32>,

    /// Message length in bytes.
    #[builder(setter(custom), field(type = "u32"))]
    pub length: u32,

    /// Transmitting node.
    #[builder(setter(into, strip_option), default)]
    pub tx_node: Option<String>,

    /// Receiving nodes.
    #[builder(setter(custom), field(type = "Vec<String>"))]
    pub rx_nodes: Vec<String>,

    /// Signals with positions in this message ordered by start bit.
    #[builder(setter(custom), field(type = "Vec<CANSignalWithPosition>"))]
    pub signals: Vec<CANSignalWithPosition>,

    #[builder(setter(custom), field(type = "HashMap<String, usize>"))]
    #[serde(skip)]
    sig_map: HashMap<String, usize>,
}

impl CANMessageBuilder {
    /// Make a [`CANMessage`] from this builder.
    ///
    /// Notes:
    ///     - Message names must be at least one character long and must contain
    ///       only ASCII letters, numbers, and underscores.
    // todo: check message ID validity and choose extended or non-extended
    pub fn build(mut self) -> Result<CANMessage, CANConstructionError> {
        // set message length in bytes
        self.length = self.signals.last().map_or(0, |s| {
            let bits = s.end() + 1;

            (bits / 8) + ((bits % 8) != 0) as u32 // ceiling integer divide
        });

        let msg = self.__build()?;

        Self::check_name_validity(&msg.name)?;

        Ok(msg)
    }

    /// Add a single signal with the next available start bit.
    /// See [`add_signal_fixed()`][CANMessageBuilder::add_signal_fixed()] for more details.
    pub fn add_signal(self, sig: CANSignal) -> Result<Self, CANConstructionError> {
        let bit = self.signals.last().map_or(0, |s| s.end() + 1);

        self.add_signal_fixed(bit, sig)
    }

    /// Add a single signal with start bit specified.
    ///
    /// Checks:
    ///  - signal name does not repeat ([`SignalNameAlreadyExists`][CANConstructionError::SignalNameAlreadyExists])
    ///  - signals are specified in order([`MessageSignalOutOfOrder`][CANConstructionError::MessageSignalsOutOfOrder])
    ///  - signals in message do not overlap ([`SignalsOverlap`][CANConstructionError::SignalsOverlap])
    ///  - signal does not extend past end of message ([`SignalWillNotFitInMessage`][CANConstructionError::SignalWillNotFitInMessage])
    pub fn add_signal_fixed(
        mut self,
        bit: u32,
        sig: CANSignal,
    ) -> Result<Self, CANConstructionError> {
        // Check that signal name is unique within this message
        if self.sig_map.contains_key(&sig.name) {
            return Err(CANConstructionError::SignalNameAlreadyExists(sig.name));
        }

        if let Some(last) = self.signals.last() {
            // Check that this signal comes after the last signal.
            if bit <= (last.start()) {
                return Err(CANConstructionError::MessageSignalsOutOfOrder(
                    sig.name,
                    bit,
                    last.name_clone(),
                    last.start(),
                ));
            }
            // Check signal ranges don't overlap
            // The signals are stored sorted by start bit in self.signals, so we
            // only need to ensure the last signal's end bit is no later than this
            // signal's start bit
            if bit <= (last.end()) {
                return Err(CANConstructionError::SignalsOverlap(
                    last.name_clone(),
                    sig.name,
                    bit,
                ));
            }
        }

        // Check signal end position is not past the end of the message
        let new = CANSignalWithPosition { bit, sig };
        if new.end() > MAX_MESSAGE_BIT {
            return Err(CANConstructionError::SignalWillNotFitInMessage(
                new.name_clone(),
                new.end(),
                MAX_MESSAGE_BIT,
            ));
        }

        self.sig_map.insert(new.name_clone(), self.signals.len());
        self.signals.push(new);

        Ok(self)
    }

    /// Add multiple signals, laying out start bits so signals are back-to-back.
    ///
    /// Convenience wrapper for [`add_signal()`][Self::add_signal].
    pub fn add_signals(
        mut self,
        sigs: impl IntoIterator<Item = CANSignal>,
    ) -> Result<Self, CANConstructionError> {
        for sig in sigs {
            self = self.add_signal(sig)?;
        }

        Ok(self)
    }

    /// Add multiple signals with start bits specified.
    ///
    /// Convenience wrapper for [`add_signal_fixed()`][Self::add_signal_fixed].
    pub fn add_signals_fixed(
        mut self,
        sigs: impl IntoIterator<Item = (u32, CANSignal)>,
    ) -> Result<Self, CANConstructionError> {
        for (bit, sig) in sigs {
            self = self.add_signal_fixed(bit, sig)?;
        }

        Ok(self)
    }

    pub fn rx_node(mut self, name: &str) -> Self {
        self.rx_nodes.push(name.into());

        self
    }

    /// Check validity of message name - it should not be empty and should
    /// contain a limited set of characters - `[a-zA-Z0-9_]`.
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
    /// Get a [builder](CANMessageBuilder).
    ///
    /// Call [.build()](CANMessageBuilder::build) to build into a [`CANMessage]`.
    pub fn builder() -> CANMessageBuilder {
        CANMessageBuilder::default()
    }

    /// Get a [signal](CANSignalWithPosition) from this message by name.
    pub fn get_sig(&self, name: &str) -> Option<&CANSignalWithPosition> {
        let &idx = self.sig_map.get(name)?;
        Some(&self.signals[idx])
    }

    /// Get transmitting node.
    pub fn tx_node(&self) -> Option<&str> {
        self.tx_node.as_deref()
    }
}

// Easy indexing of msg["signal"]. Panics if signal absent.
// (no, this can't be an Option, the Index trait doesn't allow it)
impl Index<&str> for CANMessage {
    type Output = CANSignal;

    fn index(&self, index: &str) -> &Self::Output {
        &self
            .get_sig(index)
            .unwrap_or_else(|| panic!("No signal `{index}` in message `{}`", self.name))
            .sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    fn new_msg() -> CANMessageBuilder {
        CANMessage::builder()
    }

    fn basic_msg(sigs: impl IntoIterator<Item = CANSignal>) -> CANMessage {
        new_msg()
            .name("TestMessage")
            .id(0)
            .add_signals(sigs)
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    #[should_panic(expected = "No signal `nonexistent` in message `TestMessage`")]
    fn panic_on_nonexistent_signal_index() {
        _ = basic_msg([])["nonexistent"];
    }

    #[test]
    fn basic_sig_lookup() {
        // empty
        let msg = basic_msg([]);
        assert!(matches!(msg.get_sig("sigA"), None));

        // one signal
        let msg = basic_msg([basic_sig("sigA")]);
        assert!(matches!(msg["sigA"].name.as_str(), "sigA"));
        assert!(matches!(msg.get_sig("siga"), None));

        // three signals
        let msg = basic_msg([basic_sig("sigA"), basic_sig("sigB"), basic_sig("sigC")]);
        assert!(matches!(msg["sigA"].name.as_str(), "sigA"));
        assert!(matches!(msg["sigB"].name.as_str(), "sigB"));
        assert!(matches!(msg["sigC"].name.as_str(), "sigC"));
        assert!(matches!(msg.get_sig("sigD"), None));
    }

    #[test]
    fn message_name_chars() {
        let try_msg = |name: &str| -> Result<_, CANConstructionError> {
            new_msg().name(name).id(0x10).build()
        };

        // Invalid characters
        let invalid_names = ["test!", "!!!", "test.", ".test", "."];
        for name in invalid_names {
            assert!(matches!(
                try_msg(name),
                Err(CANConstructionError::MessageNameInvalidChar(..))
            ));
        }

        // Valid names
        let valid_names = ["test", "0", "_test_", "_", "___", "THING1_THING2"];
        for name in valid_names {
            assert!(matches!(try_msg(name), Ok(_)));
        }

        // Empty name
        assert!(matches!(
            try_msg(""),
            Err(CANConstructionError::MessageNameEmpty)
        ));
    }

    #[test]
    // signals are specified in order
    // ([`MessageSignalsOutOfOrder`][CANConstructionError::MessageSignalsOutOfOrder])
    fn sigs_specified_in_order() {
        let sig1 = basic_sig("sig1");
        let sig2 = basic_sig("sig2");
        let sigs = vec![(5, sig1), (0, sig2)];

        assert!(matches!(
            new_msg()
                .name("TestMessage")
                .id(0x10)
                .add_signals_fixed(sigs),
            Err(CANConstructionError::MessageSignalsOutOfOrder(..))
        ));
    }

    #[test]
    // signal name does not repeat
    // ([`SignalNameAlreadyExists`][CANConstructionError::SignalNameAlreadyExists])
    fn unique_sig_names() {
        assert!(matches!(
            new_msg()
                .name("TestMessage")
                .id(0x10)
                .add_signal(basic_sig("sigA"))
                .unwrap()
                .add_signal(basic_sig("sigA")),
            Err(CANConstructionError::SignalNameAlreadyExists(..))
        ));
    }
}
