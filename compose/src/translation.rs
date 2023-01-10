use anyhow::{Context, Result};
use opencan_core::*;
use thiserror::Error;

use crate::ymlfmt::*;

#[derive(Error, Debug)]
enum CANCompositionError {
    #[error("Invalid directive `{0}` for enumerated value `{1}`.")]
    InvalidEnumeratedValueDirective(String, String),
}

impl YDesc {
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        for (node_name, ndesc) in self.nodes {
            Self::add_node(&mut net, &node_name, ndesc)
                .context(format!("Could not build node `{node_name}`"))?;
        }

        Ok(net)
    }

    fn add_node(net: &mut CANNetwork, node_name: &str, ndesc: YNode) -> Result<()> {
        net.add_node(node_name)?;

        let msgs = ndesc.into_messages(node_name)?;

        for msg in msgs {
            net.insert_msg(msg)?;
        }

        Ok(())
    }
}

impl YNode {
    fn into_messages(self, name: &str) -> Result<Vec<CANMessage>> {
        let mut msgs = Vec::new();

        for (msg_name, mdesc) in self.messages {
            let appended_name = format!("{name}_{msg_name}");
            let m = mdesc.into_message(&appended_name, name)?;

            msgs.push(m);
        }

        Ok(msgs)
    }
}

impl YMessage {
    fn into_message(self, msg_name: &str, node_name: &str) -> Result<CANMessage> {
        let mut can_msg = CANMessageBuilder::default()
            .name(msg_name)
            .id(self.id)
            .cycletime_ms(self.cycletime_ms)
            .tx_node(node_name);

        for (sig_name, sdesc) in self.signals {
            let start_bit = sdesc.start_bit;

            let full_sig_name = format!("{node_name}_{sig_name}");

            let sig = sdesc.into_signal(&full_sig_name).context(format!(
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

        can_msg
            .build()
            .context(format!("Could not build message `{msg_name}`"))
    }
}

impl YSignal {
    fn into_signal(self, sig_name: &str) -> Result<CANSignal> {
        let mut new_sig = CANSignal::builder()
            .name(sig_name)
            .description(self.description.clone())
            .scale(self.scale);

        for h in self.enumerated_values {
            // len should be one because every `- VALUE: val` pair is its own dict
            assert!(h.iter().len() == 1);
            let e = h.into_iter().next().unwrap();

            new_sig = match e.1 {
                YEnumeratedValue::Auto(s) => {
                    if s != "auto" {
                        return Err(
                            CANCompositionError::InvalidEnumeratedValueDirective(s, e.0).into()
                        );
                    }
                    new_sig.add_enumerated_value_inferred(e.0)?
                }
                YEnumeratedValue::Exact(v) => new_sig.add_enumerated_value(e.0, v)?,
            }
        }

        new_sig = match self.width {
            Some(w) => new_sig.width(w),
            None => new_sig.infer_width_strict()?,
        };

        new_sig
            .build()
            .context(format!("Could not build signal `{sig_name}`"))
    }
}
