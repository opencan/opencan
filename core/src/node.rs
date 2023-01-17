use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A validated description of a CAN node.
#[derive(Serialize, Deserialize)]
pub struct CANNode {
    /// Name of this node.
    pub name: String,

    /// index into parent CANNetwork messages vec
    pub(crate) messages: HashMap<String, usize>,
}

impl CANNode {
    /// Get a new `Self`.
    pub fn new(name: String) -> Self {
        Self {
            name,
            messages: HashMap::new(),
        }
    }

    /// Add message to this node. Meant to be called by a CANNetwork impl only,
    /// and must ensure that this message name is unique across the network.
    pub(crate) fn add_message(&mut self, name: &str, idx: usize) {
        assert!(!self.messages.contains_key(name));

        self.messages.insert(name.into(), idx);
    }
}
