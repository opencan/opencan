use anyhow::{Context, Result};
use can::*;

use crate::ymlfmt::*;

impl YDesc {
    pub fn into_network(self) -> Result<CANNetwork> {
        let mut net = CANNetwork::new();

        for (msg_name, msg) in self.messages {
            let sigs: Vec<CANSignal> = msg
                .signals
                .iter()
                .map(|(sig_name, sdesc)| CANSignal {
                    offset: 0,
                    name: sig_name.into(),
                    description: sdesc.description.clone(),
                    value_type: can::CANValueType::Integer(CANValueTypeInteger {
                        length: 0,
                        signed: false,
                    }),
                })
                .collect();

            let desc = CANMessageDesc {
                name: msg_name.clone(),
                id: msg.id,
                cycletime_ms: msg.cycletime_ms,
                signals: sigs,
            };

            let can_msg = CANMessage::new(desc)
                .context(format!("Could not create message `{}`", msg_name))?;
            net.insert_msg(can_msg)
                .context(format!("Could not insert message `{}`", msg_name))?;
        }
        Ok(net)
    }
}
