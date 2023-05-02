use bitvec::prelude::*;
use opencan_core::{self, CANMessage, CANSignal};

use crate::Gui;

impl Gui {
    pub fn message_id_to_opencan(&self, id: u32) -> Option<CANMessage> {
        // dbg!(id);
        self.network.message_by_id(&id).cloned()
    }

    pub fn decode_message(&self, msg: &CANMessage, data: &[u8]) -> String {
        let bits = data.view_bits::<Lsb0>();
        let mut out_pairs = vec![];

        let mut longest_sig_name = 0;

        for sigbit in &msg.signals {
            let sigraw: i64 = bits[sigbit.start() as _..=sigbit.end() as _].load();

            let len = sigbit.sig.name.len();
            if len > longest_sig_name {
                longest_sig_name = len;
            }

            out_pairs.push((
                format!("{}:", &sigbit.sig.name),
                self.decode_signal(&sigbit.sig, sigraw),
            ));
        }

        // how much space between widest signal name and decoded value?
        longest_sig_name += 4;

        format!(
            "\n{}",
            out_pairs
                .into_iter()
                .map(|(name, val)| format!("{name: <longest_sig_name$}{val}\n"))
                .collect::<String>()
        )
    }

    pub fn decode_signal(&self, signal: &CANSignal, raw: i64) -> String {
        if let Some(n) = signal.enumerated_values.get_by_right(&(raw as _)) {
            n.to_owned()
        } else if signal.scale.is_some() || signal.offset.is_some() {
            let expanded = (raw as f64 * signal.scale.unwrap_or(1.)) + signal.offset.unwrap_or(0.);
            format!("{expanded:.1}") // todo make this format precision right
        } else {
            format!("{raw}")
        }
    }
}
