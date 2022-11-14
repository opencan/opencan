use anyhow::{Context, Result};
use can::*;

use crate::ymlfmt::*;

impl YDesc {
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        for (msg_name, msg) in self.messages {
            let mut can_msg = CANMessageBuilder::default()
                .name(msg_name.clone())
                .id(msg.id)
                .cycletime_ms(msg.cycletime_ms);

            for (sig_name, sdesc) in msg.signals {
                let sig = Self::make_sig(&sig_name, sdesc).context(format!(
                    "Could not create signal `{sig_name} while building `{msg_name}`"
                ))?;

                can_msg = can_msg.add_signal(sig).context(format!(
                    "Could not add signal `{sig_name}` to message `{msg_name}`"
                ))?;
            }

            let m = can_msg
                .build()
                .context(format!("Could not create message `{msg_name}`"))?;

            net.insert_msg(m)
                .context(format!("Could not insert message `{msg_name}`"))?;
        }

        Ok(net)
    }

    fn make_sig(sig_name: &str, sdesc: YSignal) -> Result<CANSignal, CANConstructionError> {
        let mut new_sig = CANSignal::builder()
            .name(sig_name.into())
            .start_bit(sdesc.start_bit) // the answer is that start_bit should be in the MESSAGE!
            .description(sdesc.description)
            .scale(sdesc.scale);

        if let Some(w) = sdesc.width {
            new_sig = new_sig.width(w);
        } else {
            new_sig = new_sig.infer_width_strict()?;
        }

        Ok(new_sig.build()?)
    }
}
