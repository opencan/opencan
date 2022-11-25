use anyhow::{Context, Result};
use can::*;

use crate::ymlfmt::*;

impl YDesc {
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        for (msg_name, mdesc) in self.messages {
            let m = Self::make_msg(&msg_name, mdesc)?;

            net.insert_msg(m)
                .context(format!("Could not insert message `{msg_name}`"))?;
        }

        Ok(net)
    }

    fn make_msg(msg_name: &str, mdesc: YMessage) -> Result<CANMessage> {
        let mut can_msg = CANMessageBuilder::default()
            .name(msg_name)
            .id(mdesc.id)
            .cycletime_ms(mdesc.cycletime_ms);

        for (sig_name, sdesc) in mdesc.signals {
            let start_bit = sdesc.start_bit;

            let sig = Self::make_sig(&sig_name, sdesc).context(format!(
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

    fn make_sig(sig_name: &str, sdesc: YSignal) -> Result<CANSignal> {
        let mut new_sig = CANSignal::builder()
            .name(sig_name)
            .description(sdesc.description)
            .scale(sdesc.scale);

        for h in sdesc.enumerated_values {
            // len should be one because every `- VALUE: val` pair is its own dict
            assert!(h.iter().len() == 1);

            let e = h.into_iter().next().unwrap();
            match e.1 {
                YEnumeratedValue::Auto(_) => {
                    new_sig = new_sig.add_enumerated_value_inferred(e.0)?;
                }
                YEnumeratedValue::Exact(v) => {
                    new_sig = new_sig.add_enumerated_value(e.0, v)?;
                }
            }
        }

        if let Some(w) = sdesc.width {
            new_sig = new_sig.width(w);
        } else {
            new_sig = new_sig.infer_width_strict()?;
        }

        new_sig
            .build()
            .context(format!("Could not build signal `{sig_name}`"))
    }
}
