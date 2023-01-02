use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct CANNode {
    name: String,

    // index into parent CANNetwork messages vec
    messages: HashMap<String, usize>,
}

impl CANNode {
    pub fn new(name: String) -> Self {
        CANNode {
            name,
            messages: HashMap::new(),
        }
    }
}
