use indoc::formatdoc;
use opencan_core::CANMessageKind;

use crate::message_ok::MessageStatusCodegen;
use crate::node_ok::NodeStatusCodegen;
use crate::Codegen;
use crate::CodegenOutput;
use crate::Indent;
use crate::MessageCodegen;

impl<'n> Codegen<'n> {
    pub fn rx_h(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_rx_messages {
            messages += &formatdoc! {"
                /*********************************************************/
                /* RX Message: {} */
                /*********************************************************/
                ",
                msg.name
            };

            // Is this a raw message?
            if matches!(msg.kind(), CANMessageKind::Raw) {
                messages += &formatdoc! {"
                    /* --- This is a raw message. --- */

                    /*** RX Processing Function ***/
                    {};

                    /*** User RX Callback Function ***/
                    {};
                    ",
                    msg.rx_fn_decl(),
                    msg.rx_callback_fn_decl(), // there's always an RX callback for raw messages
                };

                continue;
            }

            // Not a raw message.
            messages += "\n";
            messages += &formatdoc! {"
                /*** Signal Enums ***/

                {enums}

                /*** Message Structs ***/

                {mstruct_raw}

                {mstruct}

                /*** Signal Getters ***/

                {getters}

                /*** RX Processing Function ***/

                {rx_decl};
                ",
                mstruct_raw = msg.raw_struct_def(),
                mstruct = msg.struct_def(),
                getters = msg.getter_fn_decls(),
                enums = msg.signal_enums(),
                rx_decl = msg.rx_fn_decl(),
            };

            // There should be a user callback function if the message has no cycletime.
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

            /*********************************************************/
            /* Message Health Checks */
            /*********************************************************/

            {message_checks}

            #endif
            ",
            greet = self.internal_prelude_greeting(CodegenOutput::RX_H_NAME),
            std_incl = Self::common_std_includes(),
            templates_h = CodegenOutput::TEMPLATES_H_NAME,
            rx_handler = Self::RX_HANDLER_FN_NAME,
            rx_fn_ptr = Self::RX_FN_PTR_TYPEDEF,
            rx_fn_name = Self::ID_TO_RX_FN_NAME,
            node_checks = self.node_ok_fn_decls(),
            message_checks = self.message_ok_fn_decls(),
        }
    }

    pub fn rx_c(&self) -> String {
        let mut messages = String::new();

        for msg in &self.sorted_rx_messages {
            messages += &formatdoc! {"
                /*********************************************************/
                /* RX Message: {} */
                /*********************************************************/
                ",
                msg.name
            };

            // Is this a raw message?
            if matches!(msg.kind(), CANMessageKind::Raw) {
                messages += &formatdoc! {"
                    /* --- This is a raw message. --- */

                    /*** RX Processing Function ***/
                    {}
                    ",
                    msg.rx_fn_def()
                };

                // Emit a stub if stubs are enabled.
                if self.args.rx_callback_stubs {
                    messages += &formatdoc! {"
                        /*** RX Callback Stub Function */

                        {}
                        ",
                        msg.rx_callback_fn_stub()
                    }
                }

                continue;
            }

            messages += "\n";
            messages += &formatdoc! {"
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
                mstruct_raw_name = msg.raw_struct_ty(),
                global_ident_raw = msg.global_raw_struct_ident(),
                mstruct_name = msg.struct_ty(),
                global_ident = msg.global_struct_ident(),
                timestamp = msg.rx_timestamp_ident(),
                getters = msg.getter_fn_defs(),
                rx_def = msg.rx_fn_def(),
            };

            // Emit a stub if stubs are enabled and the message has no cycletime.
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

            /*********************************************************/
            /* Message Health Checks */
            /*********************************************************/

            {message_checks}
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
            message_checks = self.message_ok_fn_defs(),
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
}
