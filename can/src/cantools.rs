use std::fmt::Display;

use indoc::formatdoc;
use textwrap::indent;

use crate::*;

pub struct CantoolsDecoder;

fn option_to_py<T: Display>(opt: &Option<T>) -> String {
    if let Some(o) = opt {
        format!("{}", o)
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
            signals.push(CantoolsDecoder::dump_signal(sig));
        }

        formatdoc!(
            "
            cantools.database.can.Message(
                name = {:?},
                frame_id = {:#x},
                length = 2,
                cycle_time = {},
                signals = [
            {}
                ]
            )
            ",
            msg.name,
            msg.id,
            option_to_py(&msg.cycletime_ms),
            indent(&signals.join("\n"), &" ".repeat(8))
        )
    }

    fn dump_signal(sig: &CANSignal) -> String {
        formatdoc!(
            "
            cantools.database.can.Signal(
                name = {:?},
                start = {},
                length = {},
                comment = {:?},
                scale = {},
                offset = {},
            ),
            ",
            sig.name,
            sig.start_bit,
            sig.width,
            option_to_py(&sig.description),
            option_to_py(&sig.scale),
            if let Some(o) = sig.offset { o } else { 0.0 },
        )
    }
}
