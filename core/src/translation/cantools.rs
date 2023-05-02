use std::fmt::Display;

use indoc::formatdoc;
use textwrap::indent;

use super::TranslationFromOpencan;
use crate::*;

/// Translation to `cantools` Python code.
pub struct CantoolsTranslator;

fn option_to_py<T: Display>(opt: &Option<T>) -> String {
    match opt {
        Some(o) => format!("{o}"),
        None => "None".into(),
    }
}

fn bool_to_py(b: bool) -> &'static str {
    if b {
        "True"
    } else {
        "False"
    }
}

impl TranslationFromOpencan for CantoolsTranslator {
    fn dump_network(net: &CANNetwork) -> String {
        let mut messages = Vec::new();
        for msg in net.iter_messages() {
            messages.push(Self::dump_message(msg));
        }

        let messages = indent(messages.join(",\n").trim(), &" ".repeat(4));
        let nodes: String = indent(
            &net.iter_nodes()
                .map(|n| format!("cantools.database.can.Node(\'{}\'),\n", n.name))
                .collect::<String>(),
            &" ".repeat(4),
        );

        formatdoc! {"
            import cantools

            messages = [
            {messages}
            ]

            nodes = [
            {nodes}
            ]

            db = cantools.database.can.Database(messages=messages, nodes=nodes)
            cantools.database.dump_file(db, 'opencan.dbc')
        "}
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
                senders = ['{}'],
                cycle_time = {},
                signals = [
            {}
                ]
            )
            ",
            msg.name,
            msg.id,
            msg.length,
            option_to_py(&msg.tx_node),
            option_to_py(&msg.cycletime),
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
                is_signed = {},
                choices = {{
            {}
                }},
            ),
            ",
            s.sig.name,
            s.start(),
            s.sig.width,
            option_to_py(&s.sig.description),
            s.sig.scale.unwrap_or(1.0),
            s.sig.offset.unwrap_or(0.0),
            bool_to_py(s.sig.twos_complement),
            indent(&Self::signal_py_choices(&s.sig), &" ".repeat(8))
        )
    }
}

impl CantoolsTranslator {
    fn signal_py_choices(s: &CANSignal) -> String {
        let mut ch: Vec<(&String, &u64)> = s.enumerated_values.iter().collect();

        ch.sort_by_key(|e| e.1);

        ch.iter()
            .map(|(s, v)| format!("{v}: {s:?},"))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
