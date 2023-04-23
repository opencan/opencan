use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::Parser;
use indoc::formatdoc;
use opencan_core::{CANMessage, CANMessageKind, CANNetwork};
use textwrap::indent;

pub mod message;
use message::MessageCodegen;

pub mod c_rx;
pub mod c_tx;
pub mod node_ok;
pub mod signal;

#[derive(Clone, Parser)]
pub struct Args {
    /// Node in the network to generate for
    pub node: String,
    /// Emit weak stub TX functions.
    #[clap(long)]
    pub tx_stubs: bool,
    /// Emit weak stub RX callback functions for messages with RX callbacks.
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

                messages // skip our own tx messages
                    .into_iter()
                    .filter(|m| m.tx_node().is_some_and(|n| n != args.node))
                    .collect()
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
