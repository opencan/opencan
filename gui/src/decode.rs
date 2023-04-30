use bitvec::prelude::*;
use opencan_core::{self, CANMessage, CANSignal};

use crate::Gui;

impl Gui {
    pub fn message_id_to_opencan(&self, id: u32) -> CANMessage {
        self.network.message_by_id(&id).unwrap().to_owned()
    }

    pub fn decode_message(&self, msg: &CANMessage, data: &[u8]) -> String {
        let bits = data.view_bits::<Lsb0>();
        let mut out = String::new();
        out += &msg.name;
        out += "(\n";

        for sigbit in &msg.signals {
            let sigraw: u64 = bits[sigbit.start() as _..=sigbit.end() as _].load();

            out += &format!("- {}\n", self.decode_signal(&sigbit.sig, sigraw));
        }

        out += ")\n";

        out
    }

    pub fn decode_signal(&self, signal: &CANSignal, raw: u64) -> String {
        let mut out = String::new();

        if signal.scale.is_some() || signal.offset.is_some() {
            let expanded = (raw as f32 * signal.scale.unwrap_or(1.)) + signal.offset.unwrap_or(0.);
            out += &format!("{}: {}", signal.name, expanded);
        } else {
            out += &format!("{}: {}", signal.name, raw);
        }

        if let Some(n) = signal.enumerated_values.get_by_right(&raw) {
            out += &format!(" ('{}')", n);
        }

        out
    }
}
