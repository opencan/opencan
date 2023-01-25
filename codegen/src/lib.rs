use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::Parser;
use indoc::formatdoc;
use opencan_core::{CANMessage, CANMessageKind, CANNetwork};
use textwrap::indent;

pub mod message;
use message::*;

pub mod signal;

#[derive(Clone, Parser)]
pub struct Args {
    /// Node in the network to generate for
    pub node: String,
    /// Emit weak stub TX functions.
    #[clap(long)]
    pub tx_stubs: bool,
    /// Emit weak stub RX callback functions for
    /// messages with RX callbacks.
    #[clap(long)]
    pub rx_callback_stubs: bool,
}

#[non_exhaustive]
pub struct CodegenOutput {
    pub callbacks_h: String,
    pub templates_h: String,
    pub rx_c: String,
    pub rx_h: String,
    pub tx_c: String,
    pub tx_h: String,
}

pub struct Codegen<'n> {
    args: Args,
    net: &'n CANNetwork,
    sorted_tx_messages: Vec<&'n CANMessage>,
    sorted_rx_messages: Vec<&'n CANMessage>,
}

trait Indent {
    fn indent(&mut self, n: usize) -> String;
}

impl Indent for String {
    fn indent(&mut self, n: usize) -> String {
        indent(self, &" ".repeat(n))
    }
}

impl Indent for &str {
    fn indent(&mut self, n: usize) -> String {
        indent(self, &" ".repeat(n))
    }
}

impl<'n> Codegen<'n> {
    const RX_FN_PTR_TYPEDEF: &str = "rx_fn_ptr";
    const ID_TO_RX_FN_NAME: &str = "CANRX_id_to_rx_fn";
    const RX_HANDLER_FN_NAME: &str = "CANRX_handle_rx";

    pub fn new(args: Args, net: &'n CANNetwork) -> Result<Self> {
        net.node_by_name(&args.node)
            .context(format!("Node `{}` not found in network.", args.node))?;

        Ok(Self {
            net,
            sorted_rx_messages: {
                let mut messages = net.rx_messages_by_node(&args.node).unwrap();

                messages.sort_by_key(|m| m.id);

                messages
            },
            sorted_tx_messages: {
                let mut messages = net.tx_messages_by_node(&args.node).unwrap();

                messages.sort_by_key(|m| m.id);

                messages
            },
            args,
        })
    }

    pub fn network_to_c(self) -> CodegenOutput {
        CodegenOutput {
            callbacks_h: self.callbacks_h(),
            templates_h: self.templates_h(),
            rx_c: self.rx_c(),
            rx_h: self.rx_h(),
            tx_c: self.tx_c(),
            tx_h: self.tx_h(),
        }
    }

    fn templates_h(&self) -> String {
        // visit all RX and TX messages, and if they're derived from a template,
        // check if we've already emitted it, and if not, emit definitions for it.

        let mut templates: HashMap<String, String> = HashMap::new();

        for message in [&self.sorted_rx_messages, &self.sorted_tx_messages]
            .into_iter()
            .flatten()
        {
            let CANMessageKind::FromTemplate(template_name) = message.kind() else {
                continue;
            };

            if templates.contains_key(template_name) {
                continue;
            }

            let template = self
                .net
                .template_message_by_name(template_name)
                .expect("template in network for FromTemplate message");

            let def = formatdoc! {"
                /*********************************************************/
                /* Message Template: {name} */
                /*********************************************************/

                /*** Template Signal Enums ***/

                {enums}

                /*** Template Message Structs ***/

                {mstruct_raw}

                {mstruct}
                ",
                name = template.name,
                enums = template.signal_enums(),
                mstruct_raw = template.raw_struct_def(),
                mstruct = template.struct_def()
            };

            templates.insert(template.name.clone(), def);
        }

        let mut templates: Vec<_> = templates.into_iter().collect();
        templates.sort();

        let templates = templates
            .into_iter()
            .map(|(_, def)| def)
            .collect::<Vec<String>>()
            .join("\n");

        formatdoc! {"
            {greet}

            #ifndef OPENCAN_TEMPLATES_H
            #define OPENCAN_TEMPLATES_H

            {std_incl}

            {templates}

            #endif
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::TEMPLATES_H_NAME),
            std_incl = Self::common_std_includes(),
        }
    }

    fn rx_h(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_rx_messages {
            messages += "\n";
            messages += &formatdoc! {"
                /*********************************************************/
                /* RX Message: {name} */
                /*********************************************************/

                /*** Signal Enums ***/

                {enums}

                /*** Message Structs ***/

                {mstruct_raw}

                {mstruct}

                /*** Signal Getters ***/

                {getters}

                /*** RX Processing Function ***/

                {rx_decl}
                ",
                name = msg.name,
                mstruct_raw = msg.raw_struct_def(),
                mstruct = msg.struct_def(),
                getters = msg.getter_fn_decls(),
                enums = msg.signal_enums(),
                rx_decl = msg.rx_fn_decl(),
            };

            if msg.cycletime.is_none() {
                messages += &formatdoc! {"

                    /*** User RX Callback Function ***/

                    {};
                    ",
                    msg.rx_callback_fn_decl()
                };
            }
        }

        let messages = messages.trim();

        formatdoc! {"
            {greet}

            #ifndef OPENCAN_RX_H
            #define OPENCAN_RX_H

            {std_incl}

            #include \"{templates_h}\"

            /*********************************************************/
            /* Primary Rx Handler Function */
            /*********************************************************/

            void {rx_handler}(uint32_t id, uint8_t * data, uint8_t len);

            /*********************************************************/
            /* ID-to-Rx-Function Lookup */
            /*********************************************************/

            typedef bool (*{rx_fn_ptr})(const uint8_t * data, uint_fast8_t len);
            {rx_fn_ptr} {rx_fn_name}(uint32_t id);

            {messages}

            /*********************************************************/
            /* Node Health Checks */
            /*********************************************************/

            {node_checks}

            #endif
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::RX_H_NAME),
            std_incl = Self::common_std_includes(),
            templates_h = CodegenOutput::TEMPLATES_H_NAME,
            rx_handler = Self::RX_HANDLER_FN_NAME,
            rx_fn_ptr = Self::RX_FN_PTR_TYPEDEF,
            rx_fn_name = Self::ID_TO_RX_FN_NAME,
            node_checks = self.node_ok_fn_decls(),
        }
    }

    fn rx_c(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_rx_messages {
            messages += "\n";
            messages += &formatdoc! {"
                /*********************************************************/
                /* RX Message: {name} */
                /*********************************************************/

                /*** Message Structs ***/

                static {mstruct_raw_name} {global_ident_raw};
                static {mstruct_name} {global_ident};

                /*** Accounting Data ***/

                static _Atomic uint64_t {timestamp};

                /*** Signal Getters ***/

                {getters}

                /*** RX Processing Function ***/

                {rx_def}
                ",
                name = msg.name,
                mstruct_raw_name = msg.raw_struct_ty(),
                global_ident_raw = msg.global_raw_struct_ident(),
                mstruct_name = msg.struct_ty(),
                global_ident = msg.global_struct_ident(),
                timestamp = msg.rx_timestamp_ident(),
                getters = msg.getter_fn_defs(),
                rx_def = msg.rx_fn_def(),
            };

            if self.args.rx_callback_stubs && msg.cycletime.is_none() {
                messages += &formatdoc! {"
                    /*** RX Callback Stub Function */

                    {}
                    ",
                    msg.rx_callback_fn_stub()
                }
            }
        }

        let messages = messages.trim();

        formatdoc! {"
            {greet}

            {std_incl}

            #include \"{rx_h}\"
            #include \"{callbacks_h}\"

            /*********************************************************/
            /* Primary Rx Handler Function */
            /*********************************************************/

            void {rx_handler}(const uint32_t id, uint8_t * const data, const uint8_t len) {{
                const {rx_fn_ptr} rx_fn = {id_to_rx_name}(id);

                if (rx_fn) {{
                    rx_fn(data, len);
                }}
            }}

            /*********************************************************/
            /* ID-to-Rx-Function Lookup */
            /*********************************************************/

            {id_to_rx_def}

            {messages}

            /*********************************************************/
            /* Node Health Checks */
            /*********************************************************/

            {node_checks}
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::RX_C_NAME),
            callbacks_h = CodegenOutput::CALLBACKS_H_NAME,
            rx_handler = Self::RX_HANDLER_FN_NAME,
            rx_fn_ptr = Self::RX_FN_PTR_TYPEDEF,
            id_to_rx_name = Self::ID_TO_RX_FN_NAME,
            rx_h = CodegenOutput::RX_H_NAME,
            std_incl = Self::common_std_includes(),
            id_to_rx_def = self.rx_id_to_decode_fn(),
            node_checks = self.node_ok_fn_defs(),
        }
    }

    fn tx_h(&self) -> String {
        let mut messages = String::new();

        // todo: struct members don't necessarily need to be _Atomic for tx
        for msg in &self.sorted_tx_messages {
            messages += &formatdoc! {"
                /*********************************************************/
                /* TX Message: {name} */
                /*********************************************************/

                /*** Signal Enums ***/

                {enums}

                /*** Message Structs ***/

                {mstruct_raw}

                {mstruct}

                /*** User-provided Populate Function ***/

                {pop_fn};

                /*** TX Processing Function ***/

                {tx_decl};

                ",
                name = msg.name,
                mstruct_raw = msg.raw_struct_def(),
                mstruct = msg.struct_def(),
                enums = msg.signal_enums(),
                pop_fn = msg.tx_populate_fn_decl(),
                tx_decl = msg.tx_fn_decl(),
            }
        }

        let messages = messages.trim();

        formatdoc! {"
            {greet}

            #ifndef OPENCAN_TX_H
            #define OPENCAN_TX_H

            {std_incl}

            #include \"{templates_h}\"

            // todo comment
            void CANTX_scheduler_1kHz(void);

            {messages}

            #endif
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::TX_H_NAME),
            std_incl = Self::common_std_includes(),
            templates_h = CodegenOutput::TEMPLATES_H_NAME,
        }
    }

    fn tx_c(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_tx_messages {
            messages += &formatdoc! {"
                /*********************************************************/
                /* TX Message: {name} */
                /*********************************************************/

                /*** TX Processing Function ***/

                {tx_def}

                ",
                name = msg.name,
                tx_def = msg.tx_fn_def(),
            };

            if self.args.tx_stubs {
                messages += &formatdoc! {"
                    /*** TX Stub Function ***/

                    {tx_stub}

                    ",
                    tx_stub = msg.tx_populate_fn_stub(),
                };
            }
        }

        let messages = messages.trim();

        formatdoc! {"
            {greet}

            {std_incl}

            #include \"{tx_h}\"
            #include \"{callbacks_h}\"


            {tx_scheduler}

            {messages}
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::TX_C_NAME),
            std_incl = Self::common_std_includes(),
            tx_h = CodegenOutput::TX_H_NAME,
            callbacks_h = CodegenOutput::CALLBACKS_H_NAME,
            tx_scheduler = self.tx_scheduler(),
        }
    }

    fn callbacks_h(&self) -> String {
        formatdoc! {"
            {}

            #ifndef OPENCAN_CALLBACKS_H
            #define OPENCAN_CALLBACKS_H

            {std_incl}

            void CAN_callback_enqueue_tx_message(const uint8_t *data, uint8_t len, uint32_t id);
            uint64_t CAN_callback_get_system_time(void);

            #endif
            ",
            self.internal_prelude_greeting(CodegenOutput::CALLBACKS_H_NAME),
            std_incl = Self::common_std_includes(),
        }
    }

    fn common_std_includes() -> String {
        formatdoc! {"
            #include <stdbool.h>
            #include <stddef.h>
            #include <stdint.h>",
        }
    }

    fn internal_prelude_greeting(&self, filename: &str) -> String {
        formatdoc! {"
            /**
             * OpenCAN CAN C Codegen - {}
             *
             * Node: {}
             *
             * spdx-license-identifier: MPL-2.0
             *
             * Generated by {} v{} @ {}
             */
            ",
            filename,
            self.args.node,
            clap::crate_name!(),
            clap::crate_version!(),
            git_version::git_version!(),
        }
    }

    /// Message ID to decode function pointer mapping.
    // todo: extended vs standard IDs?
    fn rx_id_to_decode_fn(&self) -> String {
        let mut cases = String::new();

        for msg in &self.sorted_rx_messages {
            cases += &formatdoc! {"
                case 0x{:X}U: return {};
                ",
                msg.id,
                msg.rx_fn_name()
            };
        }

        formatdoc! {"
            {dec_ptr} {name}(const uint32_t id)
            {{
                switch (id) {{
            {cases}
                    default:
                        return NULL;
                }}
            }}",
            dec_ptr = Self::RX_FN_PTR_TYPEDEF,
            name = Self::ID_TO_RX_FN_NAME,
            cases = cases.trim().indent(8),
        }
    }

    /// Tx scheduler
    fn tx_scheduler(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_tx_messages {
            let Some(cycletime) = msg.cycletime else {
                continue; // skip messages with no cycletime
            };

            messages += &formatdoc! {"
                if ((ms % {cycletime}U) == 0U) {{
                    {tx_fn}();
                }}

                ",
                tx_fn = msg.tx_fn_name(),
            };
        }

        let messages = messages.trim().indent(4);

        formatdoc! {"
            /*********************************************************/
            /* TX Scheduler */
            /*********************************************************/

            void CANTX_scheduler_1kHz(void) {{
                static uint32_t ms;
                ms++;

            {messages}
            }}"
        }
    }

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

impl CodegenOutput {
    const CALLBACKS_H_NAME: &str = "opencan_callbacks.h";
    const TEMPLATES_H_NAME: &str = "opencan_templates.h";
    const RX_C_NAME: &str = "opencan_rx.c";
    const RX_H_NAME: &str = "opencan_rx.h";
    const TX_C_NAME: &str = "opencan_tx.c";
    const TX_H_NAME: &str = "opencan_tx.h";

    pub fn as_list(&self) -> Vec<(&str, &str)> {
        [self.as_list_c(), self.as_list_h()].concat()
    }

    pub fn as_list_c(&self) -> Vec<(&str, &str)> {
        vec![(Self::RX_C_NAME, &self.rx_c), (Self::TX_C_NAME, &self.tx_c)]
    }

    pub fn as_list_h(&self) -> Vec<(&str, &str)> {
        vec![
            (Self::CALLBACKS_H_NAME, &self.callbacks_h),
            (Self::TEMPLATES_H_NAME, &self.templates_h),
            (Self::RX_H_NAME, &self.rx_h),
            (Self::TX_H_NAME, &self.tx_h),
        ]
    }
}
