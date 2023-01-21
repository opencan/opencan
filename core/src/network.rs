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
    messages: Vec<CANMessage>,

    /// Map of all template-kind CANMessage in this network.
    template_messages: HashMap<String, CANMessage>,

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
    /// Create a new (empty) network.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            messages: Vec::new(),

            template_messages: HashMap::new(),

            messages_by_name: HashMap::new(),
            messages_by_id: HashMap::new(),

            nodes_by_name: HashMap::new(),
        }
    }

    /// Get message in this network by name.
    pub fn message_by_name(&self, name: &str) -> Option<&CANMessage> {
        let &idx = self.messages_by_name.get(name)?;
        Some(&self.messages[idx])
    }

    /// Get message in this network by ID.
    pub fn message_by_id(&self, id: &u32) -> Option<&CANMessage> {
        let &idx = self.messages_by_id.get(id)?;
        Some(&self.messages[idx])
    }

    /// Insert a message into the network.
    ///
    /// Notes:
    ///     - Checks for message ID and name uniqueness.
    ///     - If a node is specified, it must already exist in the network.
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

            self.nodes[node_idx].add_tx_message(&msg.name, msg_idx);
        }

        self.messages_by_name.insert(msg.name.clone(), msg_idx);
        self.messages_by_id.insert(msg.id, msg_idx);

        self.messages.push(msg);
        Ok(())
    }

    /// Add a new node to the network.
    ///
    /// Notes:
    ///     - Checks for node name uniqueness.
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

    /// Get a node in this network by name.
    pub fn node_by_name(&self, name: &str) -> Option<&CANNode> {
        let &idx = self.nodes_by_name.get(name)?;

        Some(&self.nodes[idx])
    }

    /// Iterate over messages in this network.
    pub fn iter_messages(&self) -> std::slice::Iter<CANMessage> {
        self.messages.iter()
    }

    /// Iterate over nodes in this network.
    pub fn iter_nodes(&self) -> std::slice::Iter<CANNode> {
        self.nodes.iter()
    }

    /// Get messages transmitted by given node. Returns `None` if node does not exist.
    pub fn tx_messages_by_node(&self, name: &str) -> Option<Vec<&CANMessage>> {
        let &node_idx = self.nodes_by_name.get(name)?;
        let node = &self.nodes[node_idx];

        Some(
            node.tx_messages
                .iter()
                .map(|(_, &v)| &self.messages[v])
                .collect(),
        )
    }

    /// Get messages recieved by given node. Returns `None` if node does not exist.
    pub fn rx_messages_by_node(&self, name: &str) -> Option<Vec<&CANMessage>> {
        let &node_idx = self.nodes_by_name.get(name)?;
        let node = &self.nodes[node_idx];

        Some(
            node.rx_messages
                .iter()
                .map(|(_, &v)| &self.messages[v])
                .collect(),
        )
    }

    pub fn set_message_rx_by_node(
        &mut self,
        msg: &str,
        node: &str,
    ) -> Result<(), CANConstructionError> {
        let Some(&msg_idx) = self.messages_by_name.get(msg) else {
            return Err(CANConstructionError::MessageDoesNotExist(msg.into()));
        };

        let Some(&node_idx) = self.nodes_by_name.get(node) else {
            return Err(CANConstructionError::NodeDoesNotExist(node.into()))
        };
        let node = &mut self.nodes[node_idx];

        node.add_rx_message(msg, msg_idx);

        Ok(())
    }

    pub fn insert_template_message(&mut self, template: CANMessage) -> Result<(), CANConstructionError> {
        let name = template.name.clone();

        if self.template_messages.insert(name.clone(), template).is_some() {
            return Err(CANConstructionError::TemplateMessageNameAlreadyExists(name));
        }

        Ok(())
    }

    pub fn template_message_by_name(&self, name: &str) -> Option<&CANMessage> {
        self.template_messages.get(name)
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
