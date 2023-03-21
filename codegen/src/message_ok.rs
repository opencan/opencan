use indoc::formatdoc;

use crate::{message::MessageCodegen, Codegen, Indent};

impl<'n> Codegen<'n> {
    fn message_ok_fn_name(&self, message: &str) -> String {
        format!("CANRX_is_message_{message}_ok")
    }

    fn message_ok_fn_decl(&self, message: &str) -> String {
        format!("bool {}(void)", self.message_ok_fn_name(message))
    }

    pub fn message_ok_fn_decls(&self) -> String {
        // collect into vec
        let mut checks: Vec<_> = self
            .sorted_rx_messages
            .iter()
            .map(|m| format!("{};", self.message_ok_fn_decl(&m.name)))
            .collect();

        // sort by node name
        checks.sort();

        // collect into string with \n separators
        checks.join("\n")
    }

    pub fn message_ok_fn_defs(&self) -> String {
        const TIME_TY: &str = "uint64_t";

        let mut checks = String::new();

        for message in &self.sorted_rx_messages {
            let Some(cycletime) = message.cycletime else {
                continue; // just don't check this message
            };

            let timestamp = &formatdoc! {"
                const {TIME_TY} timestamp = {};
                ",
                message.rx_timestamp_ident()
            };
            let check = &formatdoc! {"

                timestamp != 0U && (current_time - timestamp) <= (({cycletime}U * MS_TO_US) + LATENESS_TOLERANCE_US) &&",
            };

            let timestamp = timestamp.trim().indent(4);
            let check = check.strip_suffix("&&").unwrap().trim().indent(8);

            // all together now
            checks += &formatdoc! {"
                {decl} {{
                    // Check that message has been recieved (ever) + that it's on time.
                    const {TIME_TY} current_time = CAN_callback_get_system_time();
                    const uint_fast16_t MS_TO_US = 1000U;
                    const uint_fast16_t LATENESS_TOLERANCE_US = 100U;

                {timestamp}

                    if (
                {check}
                    ) {{
                        return true;
                    }}

                    return false;
                }}\n\n",
                decl = self.message_ok_fn_decl(&message.name),
            }
        }

        checks.trim().into()
    }
}
