use anyhow::{Context, Result};
use clap::Parser;
use indoc::formatdoc;
use opencan_core::{CANMessage, CANNetwork};
use textwrap::indent;

pub mod message;
use message::*;

pub mod signal;

#[derive(Parser)]
pub struct Args {
    pub node: String,
    pub in_file: String,
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
    const DECODE_FN_PTR_TYPEDEF: &str = "decode_fn_ptr";
    const ID_TO_DECODE_FN: &str = "CANRX_id_to_decode_fn";

    pub fn new(args: Args, net: &'n CANNetwork) -> Self {
        Self {
            args,
            net,
            time: chrono::Utc::now(),
        }
    }

    pub fn network_to_c(&self) -> Result<String> {
        let mut output = String::new();

        self.net
            .node_by_name(&self.args.node)
            .context(format!("Node `{}` not found in network.", self.args.node))?;

        output += &formatdoc! {"
            {greet}

            {pre_defs}
            ",
            greet = self.internal_prelude_greeting(),
            pre_defs = Self::internal_prelude_defs(),
        };

        for msg in self.sorted_messages() {
            output += "\n";
            output += &formatdoc! {"
                /*********************************************************/
                /* Message: {name} */
                /*********************************************************/

                /*** Message Structs ***/

                {mstruct_raw}
                static {mstruct_raw_name} {global_ident_raw};

                {mstruct}
                static {mstruct_name} {global_ident};

                /*** Signal Getters ***/

                {getters}

                /*** Decode Function ***/

                {decode_fn}
                ",
                name = msg.name,
                mstruct_raw = msg.raw_struct_def(),
                mstruct_raw_name = msg.raw_struct_ty(),
                global_ident_raw = msg.global_raw_struct_ident(),
                mstruct = msg.struct_def(),
                mstruct_name = msg.struct_ty(),
                global_ident = msg.global_struct_ident(),
                getters = msg.getter_fn_defs(),
                decode_fn = msg.decode_fn_def(),
            }
        }

        output += "\n";
        output += &self.rx_id_to_decode_fn();

        Ok(output)
    }

    fn internal_prelude_defs() -> String {
        formatdoc! {"
            #include <stdbool.h>
            #include <stddef.h>
            #include <stdint.h>

            typedef bool (*{})(const uint8_t * const data, const uint_fast8_t len);
            ",
            Self::DECODE_FN_PTR_TYPEDEF,
        }
    }

    fn internal_prelude_greeting(&self) -> String {
        formatdoc! {"
            /**
             * OpenCAN CAN C Codegen - opencan_generated.c
             *
             * Node: {}
             *
             * spdx-license-identifier: MPL-2.0
             *
             * Generated by {} v{} at {}
             */
            ",
            self.args.node,
            clap::crate_name!(),
            clap::crate_version!(),
            self.time.format("%a %b %d, %T %Y %Z")
        }
    }

    fn rx_id_to_decode_fn(&self) -> String {
        let mut cases = String::new();

        for msg in self.sorted_messages() {
            cases += &formatdoc! {"
                case 0x{:X}: return {};
                ",
                msg.id,
                msg.decode_fn_name()
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
            }}
            ",
            dec_ptr = Self::DECODE_FN_PTR_TYPEDEF,
            name = Self::ID_TO_DECODE_FN,
            cases = cases.trim().indent(8),
        }
    }

    /// Get messages for our node sorted by ID
    fn sorted_messages(&self) -> Vec<&CANMessage> {
        let mut messages = self.net.messages_by_node(&self.args.node).unwrap();

        messages.sort_by(|m1, m2| m1.id.cmp(&m2.id));

        messages
    }
}
