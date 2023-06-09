use indoc::formatdoc;
use opencan_core::CANMessageKind;

use crate::Codegen;
use crate::CodegenOutput;
use crate::Indent;
use crate::MessageCodegen;

impl<'n> Codegen<'n> {
    pub fn tx_h(&self) -> String {
        let mut messages = String::new();

        // todo: struct members don't necessarily need to be _Atomic for tx
        for msg in &self.sorted_tx_messages {
            messages += &formatdoc! {"
                /*********************************************************/
                /* TX Message: {} */
                /*********************************************************/

                /*** Message ID ***/
                #define CAN_MSG_{}_ID 0x{:X}U

                ",
                msg.name,
                msg.name,
                msg.id,
            };

            // Is this a raw message?
            if matches!(msg.kind(), CANMessageKind::Raw) {
                messages += &formatdoc! {"
                    /* --- This is a raw message. --- */

                    /*** User-provided Populate Function ***/

                    {};

                    /*** TX Processing Function ***/
                    {};
                    ",
                    msg.tx_populate_fn_decl(),
                    msg.tx_fn_decl()
                };

                continue;
            }

            // Not a raw message.
            messages += &formatdoc! {"
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

    pub fn tx_c(&self) -> String {
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

    /// Tx scheduler
    fn tx_scheduler(&self) -> String {
        let mut messages = String::new();

        // We use the index for decongestion: adding the index to the current
        // time introduces a phase shift for each message.
        // This is not a perfect way of doing it, but it helps a lot.
        for (idx, msg) in self.sorted_tx_messages.iter().enumerate() {
            // skip messages with no cycletime
            let Some(cycletime) = msg.cycletime else {
                continue;
            };

            messages += &formatdoc! {"
                    if (((ms + {idx}U) % {cycletime}U) == 0U) {{
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
}
