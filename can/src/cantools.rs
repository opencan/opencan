use std::fmt::Display;

use indoc::formatdoc;
use textwrap::indent;

use crate::*;

pub struct CantoolsDecoder;

fn option_to_py<T: Display>(opt: &Option<T>) -> String {
    if let Some(o) = opt {
        format!("{o}")
    } else {
        "None".into()
    }
}

impl TranslationLayer for CantoolsDecoder {
    fn dump_network(net: &CANNetwork) -> String {
        let mut messages = Vec::new();
        for msg in &net.messages {
            messages.push(Self::dump_message(msg));
        }

        messages.join("\n")
    }

    fn dump_message(msg: &CANMessage) -> String {
        let mut signals = Vec::new();

        for sig in &msg.signals {
            signals.push(Self::dump_signal(sig));
        }

        formatdoc!(
            "
            cantools.database.can.Message(
                name = {:?},
                frame_id = {:#x},
                length = {},
                cycle_time = {},
                signals = [
            {}
                ]
            )
            ",
            msg.name,
            msg.id,
            msg.length,
            option_to_py(&msg.cycletime_ms),
            indent(&signals.join("\n"), &" ".repeat(8))
        )
    }

    fn dump_signal(s: &CANSignalWithPosition) -> String {
        formatdoc!(
            "
            cantools.database.can.Signal(
                name = {:?},
                start = {},
                length = {},
                comment = {:?},
                scale = {},
                offset = {},
                choices = {{
            {}
                }},
            ),
            ",
            s.sig.name,
            s.start(),
            s.sig.width,
            option_to_py(&s.sig.description),
            option_to_py(&s.sig.scale),
            s.sig.offset.map_or(0.0, |o| o),
            indent(&Self::signal_py_choices(&s.sig), &" ".repeat(8))
        )
    }
}

impl CantoolsDecoder {
    fn signal_py_choices(s: &CANSignal) -> String {
        let mut ch: Vec<(&String, &u64)> = s.enumerated_values.iter().collect();

        ch.sort_by(|a, b| a.1.cmp(b.1));

        ch.iter()
            .map(|(s, v)| format!("'{s}': {v},"))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
