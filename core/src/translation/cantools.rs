use std::{collections::HashSet, fmt::Display};

use indoc::formatdoc;
use textwrap::indent;

use super::TranslationFromOpencan;
use crate::*;

/// Translation to `cantools` Python code.
pub struct CantoolsTranslator<'n> {
    net: &'n CANNetwork,
}

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

impl<'n> TranslationFromOpencan for CantoolsTranslator<'n> {
    fn translate(net: &CANNetwork) -> String {
        CantoolsTranslator { net }.dump_network()
    }
}

impl<'n> CantoolsTranslator<'n> {
    pub fn new(net: &'n CANNetwork) -> Self {
        CantoolsTranslator { net }
    }

    fn dump_network(&self) -> String {
        let mut messages = Vec::new();
        for msg in self.net.iter_messages() {
            messages.push(self.dump_message(msg));
        }

        let messages = indent(messages.join(",\n").trim(), &" ".repeat(4));
        let nodes: String = indent(
            &self
                .net
                .iter_nodes()
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

    pub fn dump_message(&self, msg: &CANMessage) -> String {
        let mut signals = Vec::new();

        for sig in &msg.signals {
            signals.push(self.dump_signal(sig, msg));
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

    fn dump_signal(&self, s: &CANSignalWithPosition, msg: &CANMessage) -> String {
        // DBC files indicate rx nodes by signal, not by message. Build a list
        // of nodes that recieve this message.

        let mut rx_nodes = HashSet::new();
        for node in self.net.iter_nodes() {
            if let Some(messages) = self.net.rx_messages_by_node(&node.name) {
                if messages.iter().any(|m| m.name == msg.name) {
                    rx_nodes.insert(node.name.clone());
                }
            }
        }

        let mut rx_nodes = rx_nodes
            .into_iter()
            .map(|s| format!("'{s}'"))
            .collect::<Vec<String>>();
        rx_nodes.sort();

        formatdoc!(
            "
            cantools.database.can.Signal(
                name = {:?},
                start = {},
                receivers = [
            {}
                ],
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
            indent(&rx_nodes.join(",\n"), &" ".repeat(8)),
            s.sig.width,
            option_to_py(&s.sig.description),
            s.sig.scale.unwrap_or(1.0),
            s.sig.offset.unwrap_or(0.0),
            bool_to_py(s.sig.twos_complement),
            indent(&Self::signal_py_choices(&s.sig), &" ".repeat(8))
        )
    }
}

impl CantoolsTranslator<'_> {
    fn signal_py_choices(s: &CANSignal) -> String {
        let mut ch: Vec<(&String, &u64)> = s.enumerated_values.iter().collect();

        ch.sort_by_key(|e| e.1);

        ch.iter()
            .map(|(s, v)| format!("{v}: {s:?},"))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
