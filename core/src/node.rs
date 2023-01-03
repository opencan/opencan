use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct CANNode {
    name: String,

    // index into parent CANNetwork messages vec
    pub(crate) messages: HashMap<String, usize>,
}

impl CANNode {
    pub fn new(name: String) -> Self {
        CANNode {
            name,
            messages: HashMap::new(),
        }
    }

    /// Add message to this node. Meant to be called by a CANNetwork impl only,
    /// and must ensure that this message name is unique across the network.
    pub fn add_message(&mut self, name: &str, idx: usize) {
        assert!(!self.messages.contains_key(name));

        self.messages.insert(name.into(), idx);
    }
}
