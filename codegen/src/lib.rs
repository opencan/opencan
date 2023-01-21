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
    time: chrono::DateTime<chrono::Utc>,
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

    pub fn new(args: Args, net: &'n CANNetwork) -> Self {
        Self {
            args,
            net,
            time: chrono::Utc::now(), // date recorded now
        }
    }

    pub fn network_to_c(self) -> Result<CodegenOutput> {
        self.net
            .node_by_name(&self.args.node)
            .context(format!("Node `{}` not found in network.", self.args.node))?;

        Ok(CodegenOutput {
            callbacks_h: self.callbacks_h(),
            templates_h: self.templates_h(),
            rx_c: self.rx_c(),
            rx_h: self.rx_h(),
            tx_c: self.tx_c(),
            tx_h: self.tx_h(),
        })
    }

    fn templates_h(&self) -> String {
        // visit all RX and TX messages, and if they're derived from a template,
        // check if we've already emitted it, and if not, emit definitions for it.

        let mut templates: HashMap<String, String> = HashMap::new();

        for message in [self.sorted_rx_messages(), self.sorted_tx_messages()].concat() {
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

                /*** Template Message Structs ***/

                {mstruct_raw}

                {mstruct}
                ",
                name = template.name,
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

            {std_incl}

            {templates}
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::TEMPLATES_H_NAME),
            std_incl = Self::common_std_includes(),
        }
    }

    fn rx_h(&self) -> String {
        let mut messages = String::new();

        for msg in self.sorted_rx_messages() {
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
            /* ID-to-Rx-Function Lookup */
            /*********************************************************/

            typedef bool (*{rx_fn_ptr})(const uint8_t * data, uint_fast8_t len);
            {rx_fn_ptr} {rx_fn_name}(uint32_t id);

            {messages}

            #endif
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::RX_H_NAME),
            rx_fn_ptr = Self::RX_FN_PTR_TYPEDEF,
            rx_fn_name = Self::ID_TO_RX_FN_NAME,
            std_incl = Self::common_std_includes(),
            templates_h = CodegenOutput::TEMPLATES_H_NAME,
        }
    }

    fn rx_c(&self) -> String {
        let mut messages = String::new();

        for msg in self.sorted_rx_messages() {
            messages += "\n";
            messages += &formatdoc! {"
                /*********************************************************/
                /* RX Message: {name} */
                /*********************************************************/

                /*** Message Structs ***/

                static {mstruct_raw_name} {global_ident_raw};
                static {mstruct_name} {global_ident};

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
                getters = msg.getter_fn_defs(),
                rx_def = msg.rx_fn_def(),
            }
        }

        let messages = messages.trim();

        formatdoc! {"
            {greet}

            {std_incl}

            #include \"{callbacks_h}\"
            #include \"{rx_h}\"

            /*********************************************************/
            /* ID-to-Rx-Function Lookup */
            /*********************************************************/

            {id_to_fn}

            {messages}
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::RX_C_NAME),
            callbacks_h = CodegenOutput::CALLBACKS_H_NAME,
            rx_h = CodegenOutput::RX_H_NAME,
            std_incl = Self::common_std_includes(),
            id_to_fn = self.rx_id_to_decode_fn(),
        }
    }

    fn tx_h(&self) -> String {
        let mut messages = String::new();

        for msg in self.sorted_tx_messages() {
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

        for msg in self.sorted_tx_messages() {
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
                    tx_stub = msg.tx_stub(),
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
             * Generated by {} v{} at {}
             */
            ",
            filename,
            self.args.node,
            clap::crate_name!(),
            clap::crate_version!(),
            self.time.format("%a %b %d, %T %Y %Z")
        }
    }

    /// Message ID to decode function pointer mapping.
    // todo: extended vs standard IDs?
    fn rx_id_to_decode_fn(&self) -> String {
        let mut cases = String::new();

        for msg in self.sorted_rx_messages() {
            cases += &formatdoc! {"
                case 0x{:X}: return {};
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

        for msg in self.sorted_tx_messages() {
            messages += &formatdoc! {"
                if ((ms % {cycletime}) == 0) {{
                    {tx_fn}(data, &length);
                    CAN_callback_enqueue_tx_message(data, length, 0x{id:x});
                }}

                ",
                cycletime = msg.cycletime.expect("message to have cycletime - handle this case"),
                tx_fn = msg.tx_fn_name(),
                id = msg.id,
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

                // todo: dangerous?
                uint8_t data[8];

                // todo: we kind of don't need this, we know the length already
                uint8_t length;

            {messages}
            }}"
        }
    }

    /// Get tx messages for our node sorted by ID
    fn sorted_tx_messages(&self) -> Vec<&CANMessage> {
        let mut messages = self.net.tx_messages_by_node(&self.args.node).unwrap();

        messages.sort_by_key(|m| m.id);

        messages
    }

    /// Get rx messages for our node sorted by ID
    fn sorted_rx_messages(&self) -> Vec<&CANMessage> {
        let mut messages = self.net.rx_messages_by_node(&self.args.node).unwrap();

        messages.sort_by_key(|m| m.id);

        messages
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
