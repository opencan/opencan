use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::message::*;
use crate::node::*;

#[derive(Serialize, Deserialize)]
pub struct CANNetwork {
    /// Owning Vec of all CANNode in this network.
    nodes: Vec<CANNode>,

    /// Owning Vec of all CANMessage in this network.
    pub(crate) messages: Vec<CANMessage>,

    /// index into .messages
    #[serde(skip)]
    messages_by_name: HashMap<String, usize>,

    /// index into .messages
    #[serde(skip)]
    messages_by_id: HashMap<u32, usize>,

    #[serde(skip)]
    /// index into .nodes
    nodes_by_name: HashMap<String, usize>,
}

impl Default for CANNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl CANNetwork {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            messages: Vec::new(),

            messages_by_name: HashMap::new(),
            messages_by_id: HashMap::new(),

            nodes_by_name: HashMap::new(),
        }
    }

    pub fn message_by_name(&self, name: &str) -> Option<&CANMessage> {
        let &idx = self.messages_by_name.get(name)?;
        Some(&self.messages[idx])
    }

    pub fn message_by_id(&self, id: &u32) -> Option<&CANMessage> {
        let &idx = self.messages_by_id.get(id)?;
        Some(&self.messages[idx])
    }

    /// Insert a message into the network.
    /// Checks for message ID and name uniqueness.
    /// If a node is specified, it must already exist in the network.
    pub fn insert_msg(&mut self, msg: CANMessage) -> Result<(), CANConstructionError> {
        if self.messages_by_name.contains_key(&msg.name) {
            return Err(CANConstructionError::MessageNameAlreadyExists(msg.name));
        }

        if self.messages_by_id.contains_key(&msg.id) {
            return Err(CANConstructionError::MessageIdAlreadyExists(msg.id));
        }

        let msg_idx = self.messages.len();

        if let Some(node) = &msg.tx_node {
            let &node_idx = self
                .nodes_by_name
                .get(node)
                .ok_or_else(|| CANConstructionError::NodeDoesNotExist(node.into()))?;

            // After this point, we are making changes to the network and must
            // finish up or panic, not return in an inconsistent state.

            self.nodes[node_idx].add_message(&msg.name, msg_idx);
        }

        self.messages_by_name.insert(msg.name.clone(), msg_idx);
        self.messages_by_id.insert(msg.id, msg_idx);

        self.messages.push(msg);
        Ok(())
    }

    /// Add a new node to the network.
    /// Checks for node name uniqueness.
    pub fn add_node(&mut self, name: &str) -> Result<(), CANConstructionError> {
        if self.nodes_by_name.contains_key(name) {
            return Err(CANConstructionError::NodeAlreadyExists(name.into()));
        }

        let node = CANNode::new(name.into());

        let node_idx = self.nodes.len();
        self.nodes_by_name.insert(name.into(), node_idx);

        self.nodes.push(node);
        Ok(())
    }

    pub fn node_by_name(&self, name: &str) -> Option<&CANNode> {
        let &idx = self.nodes_by_name.get(name)?;

        Some(&self.nodes[idx])
    }

    pub fn iter_messages(&self) -> std::slice::Iter<CANMessage> {
        self.messages.iter()
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<CANNode> {
        self.nodes.iter()
    }

    pub fn messages_by_node(&self, name: &str) -> Option<Vec<&CANMessage>> {
        let &node_idx = self.nodes_by_name.get(name)?;
        let node = &self.nodes[node_idx];

        Some(
            node.messages
                .iter()
                .map(|(_, &v)| &self.messages[v])
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{CANConstructionError, CANNetwork};

    #[test]
    fn node_name_unique() {
        let mut net = CANNetwork::new();

        assert!(matches!(net.add_node("TEST"), Ok(..)));
        assert!(matches!(net.add_node("test"), Ok(..)));
        assert!(matches!(
            net.add_node("TEST"),
            Err(CANConstructionError::NodeAlreadyExists(t)) if t == "TEST"));
    }
}
