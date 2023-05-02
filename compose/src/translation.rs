//! Translation of `opencan_compose` format types ([`YDesc`], [`YNode`], ...) into
//! [`opencan_core`] types.
//!
//! We build signals/messages/nodes and ultimately hand back a [`CANNetwork`].
//! Errors originating inside `opencan_core` are bubbled up.

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use opencan_core::{translation::DbcImporter, *};

use crate::ymlfmt::*;

impl YDesc {
    /// Make a `CANNetwork` from a `YDesc` (top-level yml description).
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        // Includes
        self.process_includes(&mut net)?;

        // Bitrate
        if let Some(b) = self.bitrate {
            net.set_bitrate(b);
        }

        // Add all the templates to the network
        for tmap in &self.message_templates {
            let (name, tdesc) = unmap(tmap);

            let template = tdesc
                .to_template_message(name)
                .context(format!("Could not build template `{name}`"))?;

            net.insert_template_message(template)
                .context(format!("Could not add template `{name}` to network"))?;
        }

        // unmap all nodes into tuples
        let nodes: &Vec<_> = &self.nodes.iter().map(unmap).collect();

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
        let msgs = ndesc.to_messages(net, node_name)?;

        for msg in msgs {
            net.insert_msg(msg)?;
        }

        Ok(())
    }

    fn process_includes(&self, net: &mut CANNetwork) -> Result<()> {
        for include in &self.include {
            if include.ends_with(".dbc") {
                let dbc = std::fs::read_to_string(include)?;
                DbcImporter::import_network(dbc, net);
            }
        }

        Ok(())
    }
}

impl YNode {
    /// Make a `Vec<CANMessage>` from a `YNode`.
    fn to_messages(&self, net: &CANNetwork, name: &str) -> Result<Vec<CANMessage>> {
        let mut msgs = Vec::new();

        for m in &self.messages {
            let (msg_name, mdesc) = unmap(m);

            let appended_name = format!("{name}_{msg_name}");
            let msg = mdesc.to_message(net, &appended_name, name)?;

            msgs.push(msg);
        }

        Ok(msgs)
    }
}

impl YMessageTemplate {
    /// Make a template `CANMessage` from a `YMessageTemplate`.
    fn to_template_message(&self, name: &str) -> Result<CANMessage> {
        let mut msg = CANMessage::template().name(name);

        // cycletime
        msg = msg.cycletime(self.cycletime);

        // Add signals
        msg = YMessage::add_signals_to_message_builder(msg, &self.signals, "")?;

        let msg = msg.build()?;
        Ok(msg)
    }
}

impl YMessage {
    /// Make a `CANMessage` from a `YMessage`.
    fn to_message(&self, net: &CANNetwork, msg_name: &str, node_name: &str) -> Result<CANMessage> {
        if let Some(template_name) = &self.from_template {
            // Make sure there is no signals field
            if self.signals.is_some() {
                return Err(anyhow!("Message {msg_name} inherits signals from template `{template_name}` and cannot specify a `signals:` field."));
            }

            // Find template
            let template = net
                .template_message_by_name(template_name)
                .context(format!("No template named `{template_name}` in network."))?;

            // Instantiate template
            let signal_prefix = format!("{node_name}_");
            let msg = template.template_instance(
                msg_name,
                self.id,
                &signal_prefix,
                self.cycletime,
                Some(node_name),
            )?;

            return Ok(msg);
        }

        // If we don't have a signals field, make a raw message
        let Some(signals) = &self.signals else {
            return Ok(CANMessage::new_raw(msg_name, self.id, self.cycletime, Some(node_name)));
        };

        // First, make a CANMessageBuilder.
        let mut can_msg = CANMessageBuilder::default()
            .name(msg_name)
            .id(self.id)
            .cycletime(self.cycletime)
            .tx_node(node_name);

        // Add signals
        can_msg = Self::add_signals_to_message_builder(can_msg, signals, &format!("{node_name}_"))
            .context(format!(
                "Could not populate signals for message `{msg_name}`"
            ))?;

        // Build message and return
        can_msg
            .build()
            .context(format!("Could not build message `{msg_name}`"))
    }

    fn add_signals_to_message_builder(
        mut message: CANMessageBuilder,
        signals: &Vec<HashMap<String, YSignal>>,
        signal_prefix: &str,
    ) -> Result<CANMessageBuilder> {
        // For each signal, make the YSignal into a CANSignal, and add it to the message
        for s in signals {
            let (sig_name, sdesc) = unmap(s);

            let start_bit = sdesc.start_bit;
            let full_sig_name = format!("{signal_prefix}{sig_name}");

            let sig = sdesc
                .to_signal(&full_sig_name)
                .context(format!("Could not create signal `{sig_name}`."))?;

            message = match start_bit {
                Some(bit) => message.add_signal_fixed(bit, sig),
                None => message.add_signal(sig),
            }
            .context(format!("Could not add signal `{sig_name}`."))?;
        }

        Ok(message)
    }
}

impl YSignal {
    /// Turn a `YSignal` into a `CANSignal`.
    fn to_signal(&self, sig_name: &str) -> Result<CANSignal> {
        // First, make a CANSignalBuilder.
        let mut new_sig = CANSignal::builder()
            .name(sig_name)
            .description(self.description.clone())
            .twos_complement(self.twos_complement)
            .scale(self.scale)
            .offset(self.offset);

        // Translate each enumerated value
        for h in &self.enumerated_values {
            new_sig = match h {
                YEnumeratedValue::Auto(s) => new_sig.add_enumerated_value_inferred(s)?,
                YEnumeratedValue::Exact(map) => {
                    let (name, &val) = unmap(map);
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
