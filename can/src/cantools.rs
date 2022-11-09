use indoc::formatdoc;
use textwrap::indent;

use crate::*;

pub struct CantoolsDecoder;

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
                name = {},
                signals = [
                {}
                ]
        ",
            msg.name,
            indent(&signals.join("\n"), "    ")
        )
    }

    fn dump_signal(sig: &CANSignal) -> String {
        formatdoc!(
            "
            cantools.database.can.Signal(
                    name = {},
                ),
        ",
            sig.name
        )
    }
}
