//! Translation of `opencan_compose` format types ([`YDesc`], [`YNode`], ...) into
//! [`opencan_core`] types.
//!
//! We build signals/messages/nodes and ultimately hand back a [`CANNetwork`].
//! Errors originating inside `opencan_core` are bubbled up.

use anyhow::{Context, Result};
use opencan_core::*;

use crate::ymlfmt::*;

impl YDesc {
    /// Make a `CANNetwork` from a `YDesc` (top-level yml description).
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        // unmap all nodes into tuples
        let nodes: &Vec<_> = &self.nodes.iter().map(unmap_ref).collect();

        // Add all the nodes to the network
        for (name, _) in nodes {
            net.add_node(name)?;
        }

        // Add all the messages in each node to the network
        for (name, ndesc) in nodes {
            Self::add_node_msgs(&mut net, name, ndesc)
                .context(format!("Could not build node `{name}`"))?;
        }

        // Fill in rx for each node
        for (name, ndesc) in nodes {
            // the rx field is either a directive like `rx: "*"` or a list of messages
            match &ndesc.rx {
                RxListOrDirective::List(list) => {
                    for rx in list {
                        net.set_message_rx_by_node(rx, name)
                            .context(format!("Could not add rx message `{rx}` to node `{name}`"))?;
                    }
                }
                RxListOrDirective::Directive(d) => {
                    match d {
                        RxDirective::Everything => {
                            // collect all the message names in the network
                            let messages: Vec<String> =
                                net.iter_messages().map(|m| m.name.clone()).collect();

                            // add each message to the node
                            for msg in &messages {
                                net.set_message_rx_by_node(msg, name)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(net)
    }

    /// Add contents of a `YNode` to a given network (doesn't add node itself)
    fn add_node_msgs(net: &mut CANNetwork, node_name: &str, ndesc: &YNode) -> Result<()> {
        let msgs = ndesc.to_messages(node_name)?;

        for msg in msgs {
            net.insert_msg(msg)?;
        }

        Ok(())
    }
}

impl YNode {
    /// Make a `Vec<CANMessage>` from a `YNode`.
    fn to_messages(&self, name: &str) -> Result<Vec<CANMessage>> {
        let mut msgs = Vec::new();

        for m in &self.messages {
            let (msg_name, mdesc) = unmap_ref(m);

            let appended_name = format!("{name}_{msg_name}");
            let msg = mdesc.to_message(&appended_name, name)?;

            msgs.push(msg);
        }

        Ok(msgs)
    }
}

impl YMessage {
    /// Make a `CANMessage` from a `YMessage`.
    fn to_message(&self, msg_name: &str, node_name: &str) -> Result<CANMessage> {
        // First, make a CANMessageBuilder.
        let mut can_msg = CANMessageBuilder::default()
            .name(msg_name)
            .id(self.id)
            .cycletime(self.cycletime)
            .tx_node(node_name);

        // For each signal, make the YSignal into a CANSignal, and add it to the message.
        for s in &self.signals {
            let (sig_name, sdesc) = unmap_ref(s);

            let start_bit = sdesc.start_bit;
            let full_sig_name = format!("{node_name}_{sig_name}");

            let sig = sdesc.to_signal(&full_sig_name).context(format!(
                "Could not create signal `{sig_name}` while composing message `{msg_name}`"
            ))?;

            can_msg = match start_bit {
                Some(bit) => can_msg.add_signal_fixed(bit, sig),
                None => can_msg.add_signal(sig),
            }
            .context(format!(
                "Could not add signal `{sig_name}` to message `{msg_name}`"
            ))?;
        }

        // Build message and return
        can_msg
            .build()
            .context(format!("Could not build message `{msg_name}`"))
    }
}

impl YSignal {
    /// Turn a `YSignal` into a `CANSignal`.
    fn to_signal(&self, sig_name: &str) -> Result<CANSignal> {
        // First, make a CANSignalBuilder.
        let mut new_sig = CANSignal::builder()
            .name(sig_name)
            .description(self.description.clone())
            .scale(self.scale);

        // Translate each enumerated value
        for h in &self.enumerated_values {
            new_sig = match h {
                YEnumeratedValue::Auto(s) => new_sig.add_enumerated_value_inferred(s)?,
                YEnumeratedValue::Exact(map) => {
                    let (name, &val) = unmap_ref(map);
                    new_sig.add_enumerated_value(name, val)?
                }
            };
        }

        // Either specify the width or infer it
        new_sig = match self.width {
            Some(w) => new_sig.width(w),
            None => new_sig.infer_width_strict()?,
        };

        // Build and return
        new_sig
            .build()
            .context(format!("Could not build signal `{sig_name}`"))
    }
}
