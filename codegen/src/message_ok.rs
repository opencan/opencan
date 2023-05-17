use indoc::formatdoc;
use opencan_core::CANMessage;

use crate::{message::MessageCodegen, Codegen, Indent};

pub trait MessageStatusCodegen {
    /// Name of the `message_ok` function for the given RX message.
    fn message_ok_fn_name(&self, message: &CANMessage) -> String;
    /// Declaration of the `message_ok` function for the given RX message.
    fn message_ok_fn_decl(&self, message: &CANMessage) -> String;
    /// Definition of the `message_ok` function for the given RX message.
    fn message_ok_fn_def(&self, message: &CANMessage) -> String;
    /// Declarations of the `message_ok` functions for all of our RX messages.
    fn all_rx_messages_ok_fn_decls(&self) -> String;
    /// Definitions of the `message_ok` functions for all of our RX messages.
    fn all_rx_messages_ok_fn_defs(&self) -> String;
}

impl MessageStatusCodegen for Codegen<'_> {
    fn message_ok_fn_name(&self, message: &CANMessage) -> String {
        format!("CANRX_is_message_{}_ok", message.name)
    }

    fn message_ok_fn_decl(&self, message: &CANMessage) -> String {
        let Some(_) = message.cycletime else {
            return "".into();
        };
        format!("bool {}(void)", self.message_ok_fn_name(message))
    }

    fn message_ok_fn_def(&self, message: &CANMessage) -> String {
        let Some(cycletime) = message.cycletime else {
            return "".into();
        };

        const TIME_TY: &str = "uint64_t";

        let timestamp = formatdoc! {"
            const {TIME_TY} timestamp = {};
            ",
            message.rx_timestamp_ident()
        };
        let time_check = formatdoc! {"
            timestamp != 0U && (current_time - timestamp) <= (({cycletime}U * MS_TO_US) + LATENESS_TOLERANCE_US)",
        };

        let timestamp = timestamp.trim().indent(4);

        // all together now
        let check = formatdoc! {"
            {decl} {{
                // Check that message has been recieved (ever) + that it's on time.
                const {TIME_TY} current_time = CAN_callback_get_system_time();
                const uint_fast16_t MS_TO_US = 1000U;
                const uint_fast16_t LATENESS_TOLERANCE_US = 100U;

            {timestamp}

                if (
                    {time_check}
                ) {{
                    return true;
                }}

                return false;
            }}\n\n",
            decl = self.message_ok_fn_decl(&message),
        };

        check.trim().into()
    }

    fn all_rx_messages_ok_fn_decls(&self) -> String {
        // collect into vec
        let mut checks: Vec<_> = self
            .sorted_rx_messages
            .iter()
            .map(|message| format!("{};", self.message_ok_fn_decl(message)))
            .collect();

        // sort by node name
        checks.sort();

        // collect into string with \n separators
        checks.join("\n")
    }

    fn all_rx_messages_ok_fn_defs(&self) -> String {
        let mut checks = self
            .sorted_rx_messages
            .iter()
            .map(|msg| self.message_ok_fn_def(msg))
            .collect::<Vec<_>>();

        // sort by message name
        checks.sort();

        // collect into string with \n\n separators
        checks
            .into_iter()
            .map(|def| def + "\n\n")
            .collect::<String>()
            .trim()
            .into()
    }
}
