use std::collections::HashMap;

use crate::error::*;
use crate::message::*;

pub struct CANNetwork {
    messages: Vec<CANMessage>,

    messages_by_name: HashMap<String, usize>,
    messages_by_id: HashMap<u32, usize>,
}

impl Default for CANNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl CANNetwork {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),

            messages_by_name: HashMap::new(),
            messages_by_id: HashMap::new(),
        }
    }

    pub fn message_by_name(&self, name: &str) -> Option<&CANMessage> {
        let idx = self.messages_by_name.get(name)?;
        Some(self.messages.get(*idx).unwrap())
    }

    pub fn message_by_id(&self, id: &u32) -> Option<&CANMessage> {
        let idx = self.messages_by_id.get(id)?;
        Some(self.messages.get(*idx).unwrap())
    }

    /// Create and add a message to the network.
    /// Checks for message ID and name uniqueness.
    pub fn add_msg(&mut self, msg: CANMessageDesc) -> Result<(), CANConstructionError> {
        if self.messages_by_name.get(&msg.name).is_some() {
            return Err(CANConstructionError::MessageNameAlreadyExists(msg.name));
        }

        if self.messages_by_id.get(&msg.id).is_some() {
            return Err(CANConstructionError::MessageIdAlreadyExists(msg.id));
        }

        let msg_idx = self.messages.len();
        self.messages_by_name.insert(msg.name.clone(), msg_idx);
        self.messages_by_id.insert(msg.id, msg_idx); // check dup again?

        self.messages.push(CANMessage::new(msg)?);

        Ok(())
    }
}
