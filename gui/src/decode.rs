use bitvec::prelude::*;
use opencan_core::{self, CANMessage, CANSignal};

use crate::Gui;

pub trait RawSignalValue {
    fn to_u64(&self) -> u64;
    fn to_f64(&self) -> f64;
    fn to_string(&self) -> String;
}

impl RawSignalValue for u64 {
    fn to_u64(&self) -> u64 {
        *self
    }

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl RawSignalValue for i64 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl Gui {
    pub fn message_id_to_opencan(&self, id: u32) -> Option<CANMessage> {
        // dbg!(id);
        self.network.as_ref()?.message_by_id(&id).cloned()
    }

    pub fn decode_message(&self, msg: &CANMessage, data: &[u8]) -> String {
        let bits = data.view_bits::<Lsb0>();
        let mut out_pairs = vec![];

        let mut longest_sig_name = 0;

        for sigbit in &msg.signals {
            let len = sigbit.sig.name.len();
            if len > longest_sig_name {
                longest_sig_name = len;
            }

            match sigbit.sig.twos_complement {
                true => {
                    let sigraw: i64 = bits[sigbit.start() as _..=sigbit.end() as _].load();

                    out_pairs.push((
                        format!("{}:", &sigbit.sig.name),
                        self.decode_signal(&sigbit.sig, sigraw),
                    ));
                }
                false => {
                    let sigraw: u64 = bits[sigbit.start() as _..=sigbit.end() as _].load();

                    out_pairs.push((
                        format!("{}:", &sigbit.sig.name),
                        self.decode_signal(&sigbit.sig, sigraw),
                    ));
                }
            }
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

    pub fn decode_signal<R>(&self, signal: &CANSignal, raw: R) -> String
    where
        R: RawSignalValue,
    {
        if let Some(n) = signal.enumerated_values.get_by_right(&(raw.to_u64())) {
            n.to_owned()
        } else if signal.scale.is_some() || signal.offset.is_some() {
            let expanded =
                (raw.to_f64() * signal.scale.unwrap_or(1.)) + signal.offset.unwrap_or(0.);
            format!("{expanded:.3}") // todo make this format precision right
        } else {
            raw.to_string()
        }
    }
}
