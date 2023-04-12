use std::collections::HashMap;

use indoc::formatdoc;

use crate::{message::MessageCodegen, Codegen, Indent};

pub trait NodeStatusCodegen {
    /// Name of the `node_ok` function for the given node.
    fn node_ok_fn_name(&self, node: &str) -> String;
    /// Declaration of the `node_ok` function for the given node.
    fn node_ok_fn_decl(&self, node: &str) -> String;
    /// Definition of the `node_ok` function for the given node.
    fn node_ok_fn_def(&self, node: &str) -> String;
    /// Declarations of the `node_ok` functions for all nodes we RX messages from.
    fn node_ok_fn_decls(&self) -> String;
    /// Definitions of the `node_ok` functions for all nodes we RX messages from.
    fn node_ok_fn_defs(&self) -> String;
}

impl NodeStatusCodegen for Codegen<'_> {
    fn node_ok_fn_name(&self, node: &str) -> String {
        format!("CANRX_is_node_{node}_ok")
    }

    fn node_ok_fn_decl(&self, node: &str) -> String {
        format!("bool {}(void)", self.node_ok_fn_name(node))
    }

    fn node_ok_fn_def(&self, node: &str) -> String {
        const TIME_TY: &str = "uint64_t";

        let mut timestamps = String::new();
        let mut checks = String::new();

        for message in &self.sorted_rx_messages {
            let tx_node = message.tx_node().expect("message to have tx node");
            if tx_node != node {
                continue;
            }

            let Some(cycletime) = message.cycletime else {
                continue; // just don't check this message
            };

            timestamps += &formatdoc! {"
                const {TIME_TY} timestamp_{} = {};
                ",
                message.name,
                message.rx_timestamp_ident()
            };
            checks += &formatdoc! {"

                timestamp_{name} != 0U && (current_time - timestamp_{name}) <= (({cycletime}U * MS_TO_US) + LATENESS_TOLERANCE_US) &&",
                name = message.name,
            }
        }

        // no checks for this node, just make a dummy that always returns true
        if checks.is_empty() {
            return formatdoc! {"
                {decl} {{
                    // No messages recieved from node `{node}` with a cycletime.
                    return true;
                }}",
                decl = self.node_ok_fn_decl(node)
            };
        }

        let timestamps = timestamps.trim().indent(4);
        let checks = checks.strip_suffix("&&").unwrap().trim().indent(8);

        // all together now
        formatdoc! {"
            {decl} {{
                // Check that each message has been recieved (ever) + that it's on time.
                const {TIME_TY} current_time = CAN_callback_get_system_time();
                const uint_fast16_t MS_TO_US = 1000U;
                const uint_fast16_t LATENESS_TOLERANCE_US = 100U;

            {timestamps}

                if (
            {checks}
                ) {{
                    return true;
                }}

                return false;
            }}",
            decl = self.node_ok_fn_decl(node)
        }
    }

    fn node_ok_fn_decls(&self) -> String {
        let mut checks: HashMap<String, String> = HashMap::new();

        for msg in &self.sorted_rx_messages {
            let node = msg.tx_node().expect("Message to have tx node");
            if checks.contains_key(node) {
                continue;
            }

            checks.insert(node.into(), format!("{};", self.node_ok_fn_decl(node)));
        }

        // collect into vec
        let mut checks: Vec<_> = checks.into_iter().collect();

        // sort by node name
        checks.sort();

        // collect into string with \n separators
        checks
            .into_iter()
            .map(|(_, def)| def + "\n")
            .collect::<String>()
            .trim()
            .into()
    }

    fn node_ok_fn_defs(&self) -> String {
        let mut checks: HashMap<String, String> = HashMap::new();

        for msg in &self.sorted_rx_messages {
            let node = msg.tx_node().expect("Message to have tx node");
            if checks.contains_key(node) {
                continue;
            }

            checks.insert(node.into(), self.node_ok_fn_def(node));
        }

        // collect into vec
        let mut checks: Vec<_> = checks.into_iter().collect();

        // sort by node name
        checks.sort();

        // collect into string with \n\n separators
        checks
            .into_iter()
            .map(|(_, def)| def + "\n\n")
            .collect::<String>()
            .trim()
            .into()
    }
}
