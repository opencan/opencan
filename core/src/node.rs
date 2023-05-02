use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A validated description of a CAN node.
#[derive(Debug, Serialize, Deserialize)]
pub struct CANNode {
    /// Name of this node.
    pub name: String,

    /// index into parent CANNetwork messages vec
    pub(crate) tx_messages: HashMap<String, usize>,

    /// index into parent CANNetwork messages vec
    pub(crate) rx_messages: HashMap<String, usize>,
}

impl CANNode {
    /// Get a new `Self`.
    pub fn new(name: String) -> Self {
        Self {
            name,
            tx_messages: HashMap::new(),
            rx_messages: HashMap::new(),
        }
    }

    /// Add tx message to this node. Meant to be called by a CANNetwork impl only,
    /// and must ensure that this message name is unique across the network.
    pub(crate) fn add_tx_message(&mut self, name: &str, idx: usize) {
        assert!(!self.tx_messages.contains_key(name));

        self.tx_messages.insert(name.into(), idx);
    }

    /// Add rx message to this node. Meant to be called by a CANNetwork impl only,
    /// and must ensure that this message name is unique across the network.
    pub(crate) fn add_rx_message(&mut self, name: &str, idx: usize) {
        assert!(!self.rx_messages.contains_key(name));

        self.rx_messages.insert(name.into(), idx);
    }
}
