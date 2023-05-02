use std::collections::HashMap;

use can_dbc::AttributeValuedForObjectType::MessageDefinitionAttributeValue;
use can_dbc::ByteOrder;

use crate::{CANMessage, CANNetwork, CANSignal, TranslationToOpencan};

pub struct DbcImporter {
    dbc: can_dbc::DBC,
}

impl TranslationToOpencan for DbcImporter {
    fn import_network(input: String) -> crate::CANNetwork {
        let import = Self {
            dbc: can_dbc::DBC::try_from(input.as_str()).unwrap(),
        };

        dbg!(&import.dbc);

        let mut net = CANNetwork::new();

        // wtf here
        // Add all the nodes to the network
        for node in &import.dbc.nodes().iter().next().unwrap().0 {
            net.add_node(node).unwrap();
        }

        // Add all the messages in each node to the network
        for dbc_msg in import.dbc.messages() {
            let message_id = *dbc_msg.message_id();

            let mut msg = CANMessage::builder()
                .name(dbc_msg.message_name())
                .id(message_id.0);

            // tx node
            match dbc_msg.transmitter() {
                can_dbc::Transmitter::NodeName(node) => msg = msg.tx_node(node),
                can_dbc::Transmitter::VectorXXX => todo!("support for anonymous tx node"),
            }

            // signals
            let mut opencan_signals: Vec<_> = dbc_msg
                .signals()
                .iter()
                .map(|dbc_signal| {
                    (
                        dbc_signal.start_bit as u32,
                        import.translate_signal(dbc_msg, dbc_signal),
                    )
                })
                .collect();

            opencan_signals.sort_by_key(|s| s.0);
            msg = msg.add_signals_fixed(opencan_signals).unwrap();

            // cycletime
            let cycletime = import.dbc.attribute_values().iter().find_map(|a| {
                if a.attribute_name() != "GenMsgCycleTime" {
                    return None;
                }

                if let MessageDefinitionAttributeValue(id, v) = a.attribute_value() {
                    if *id != message_id {
                        return None;
                    }

                    let t = match v.as_ref().unwrap() {
                        can_dbc::AttributeValue::AttributeValueU64(t) => *t as u32,
                        can_dbc::AttributeValue::AttributeValueI64(t) => *t as u32,
                        can_dbc::AttributeValue::AttributeValueF64(t) => *t as u32,
                        can_dbc::AttributeValue::AttributeValueCharString(t) => {
                            panic!("Didn't expect GenMsgCycleTime to be AttributeValueCharString ('{t}').")
                        }
                    };

                    if t != 0 {
                        return Some(t);
                    }
                }

                None
            });

            msg = msg.cycletime(cycletime);

            // insert message into network
            net.insert_msg(msg.build().unwrap()).unwrap();
        }

        dbg!(&net);

        net
    }
}

impl DbcImporter {
    fn translate_signal(
        &self,
        dbc_msg: &can_dbc::Message,
        dbc_signal: &can_dbc::Signal,
    ) -> CANSignal {
        let &message_id = dbc_msg.message_id();
        let signal_name = dbc_signal.name();

        let mut sig = CANSignal::builder()
            .name(dbc_signal.name())
            .width(dbc_signal.signal_size as _);

        // twos complement?
        if matches!(dbc_signal.value_type(), can_dbc::ValueType::Signed) {
            sig = sig.twos_complement(true);
        }

        // endianness
        assert!(
            matches!(dbc_signal.byte_order(), ByteOrder::LittleEndian),
            "OpenCAN doesn't support big-endian signals yet."
        );

        // scale
        if dbc_signal.factor != 1.0 {
            sig = sig.scale(Some(dbc_signal.factor));
        }

        // offset
        if dbc_signal.offset != 0.0 {
            sig = sig.offset(Some(dbc_signal.offset));
        }

        // description
        if let Some(comment) = self.dbc.signal_comment(message_id, signal_name) {
            sig = sig.description(Some(comment.to_owned()));
        }

        // emumerated values
        if let Some(d) = self
            .dbc
            .value_descriptions_for_signal(message_id, signal_name)
        {
            let mut enumerated_values: Vec<(String, u64)> = Vec::new();

            for val_desc in d {
                // unfortunately some people do insane things with their value descriptions.
                // we are going to normalize these names and prevent collisions.
                let name = val_desc.b();

                // map naughty characters to _
                let normalized_name: String = name
                    .to_ascii_uppercase()
                    .chars()
                    .map(|c| match c {
                        'A'..='Z' | '0'..='9' => c,
                        _ => '_',
                    })
                    .collect();

                // trim trailing/leading '_'
                let normalized_name = normalized_name.trim_matches('_');

                // get the value
                let value = val_desc.a();
                if value.fract() != 0.0 {
                    panic!("Expected integer value description!");
                }

                // push
                enumerated_values.push((normalized_name.into(), *value as _));
            }

            // find duplicate names
            let mut occurences: HashMap<String, u64> = HashMap::new();
            for val in &enumerated_values {
                *occurences.entry(val.0.clone()).or_insert(0) += 1;
            }

            // actually add the enumerated values
            for val in enumerated_values {
                let name = if occurences[&val.0] > 1 {
                    // making unique names if there was more than one occurrence
                    format!("{}_{}", val.1, &val.0)
                } else {
                    val.0
                };

                let val = val.1;

                sig = sig.add_enumerated_value(&name, val).unwrap();
            }
        }

        sig.build().unwrap()
    }
}
