use anyhow::{Context, Result};
use can::*;

use crate::ymlfmt::*;

impl YDesc {
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        for (msg_name, msg) in self.messages {
            let mut sigs = Vec::new();

            for (sig_name, sdesc) in msg.signals {
                let new_sig =
                    CANSignal::new(0, sig_name.clone(), sdesc.width, sdesc.description.clone())
                        .context(format!(
                            "Could not create signal `{sig_name}` in message `{msg_name}`"
                        ))?;

                sigs.push(new_sig);
            }

            let desc = CANMessageDesc {
                name: msg_name.clone(),
                id: msg.id,
                cycletime_ms: msg.cycletime_ms,
                signals: sigs,
            };

            let can_msg =
                CANMessage::new(desc).context(format!("Could not create message `{msg_name}`"))?;

            net.insert_msg(can_msg)
                .context(format!("Could not insert message `{msg_name}`"))?;
        }
        Ok(net)
    }
}
