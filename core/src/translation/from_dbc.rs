use crate::{CANMessage, CANNetwork, CANSignal, TranslationToOpencan};

pub struct DbcImporter;

impl TranslationToOpencan for DbcImporter {
    fn import_network(input: String) -> crate::CANNetwork {
        let dbc = can_dbc::DBC::try_from(input.as_str()).unwrap();

        let mut net = CANNetwork::new();

        // wtf here
        // Add all the nodes to the network
        for node in &dbc.nodes().iter().next().unwrap().0 {
            net.add_node(&node).unwrap();
        }

        // Add all the messages in each node to the network
        for dbc_msg in dbc.messages() {
            let mut msg = CANMessage::builder()
                .name(dbc_msg.message_name())
                .id(dbc_msg.message_id().0);

            let mut opencan_signals: Vec<_> = dbc_msg
                .signals()
                .iter()
                .map(|dbc_signal| {
                    (
                        dbc_signal.start_bit as u32,
                        Self::translate_signal(dbc_signal),
                    )
                })
                .collect();

            opencan_signals.sort_by_key(|s| s.0);

            msg = msg.add_signals_fixed(opencan_signals).unwrap();

            net.insert_msg(msg.build().unwrap()).unwrap();
        }

        dbg!(net);

        todo!()
    }
}

impl DbcImporter {
    fn translate_signal(dbc_signal: &can_dbc::Signal) -> CANSignal {
        let sig = CANSignal::builder()
            .name(dbc_signal.name())
            .width(dbc_signal.signal_size as _);

        sig.build().unwrap()
    }
}
