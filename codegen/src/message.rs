use indoc::formatdoc;
use opencan_core::{CANMessage, CANMessageKind};

use crate::{signal::*, Indent};

pub trait MessageCodegen {
    /// C type for this message's unpacked + decoded data struct.
    fn struct_ty(&self) -> String;
    /// Definition of this message's unpacked + decoded data struct.
    fn struct_def(&self) -> String;
    /// Identifier for the global unpacked + decoded data struct of type [`.struct_ty()`](Self::struct_ty()).
    fn global_struct_ident(&self) -> String;

    /// C type for this message's unpacked raw data struct.
    fn raw_struct_ty(&self) -> String;
    /// Definition of this message's unpacked raw data struct.
    fn raw_struct_def(&self) -> String;
    /// Identifier for the global unpacked raw data struct of type [`.raw_struct_ty()`](Self::raw_struct_ty()).
    fn global_raw_struct_ident(&self) -> String;

    /// Name of the RX handler function for this message.
    fn rx_fn_name(&self) -> String;
    /// Declaration of the RX handler function for this message.
    fn rx_fn_decl(&self) -> String;
    /// Definition of the RX handler function for this message.
    fn rx_fn_def(&self) -> String;

    /// Name of the TX handler function for this message.
    fn tx_fn_name(&self) -> String;
    /// Declaration of the TX handler function for this messsage.
    fn tx_fn_decl(&self) -> String;
    /// Definition of the TX handler function for this message.
    fn tx_fn_def(&self) -> String;
    /// Name of the TX user populate callback for this message.
    fn tx_populate_fn_name(&self) -> String;
    /// Declaration of the TX user populate function for this message.
    fn tx_populate_fn_decl(&self) -> String;

    /// Declarations of the signal getter functions for this message.
    fn getter_fn_decls(&self) -> String;
    /// Definitions of the signal getter functions for this message.
    fn getter_fn_defs(&self) -> String;
    /// Enumerations for all signals that have them in this message.
    fn signal_enums(&self) -> String;

    /// Fix up signal name within structs for template-derived messages.
    fn normalize_struct_signal_name(&self, name: &str) -> String;

    /// Stub (empty) tx function for tests.
    fn tx_stub(&self) -> String;
}

impl MessageCodegen for CANMessage {
    fn struct_ty(&self) -> String {
        match self.kind() {
            CANMessageKind::Independent => {
                format!("struct CAN_Message_{}", self.name)
            }
            CANMessageKind::Template => {
                format!("struct CAN_TMessage_{}", self.name)
            }
            CANMessageKind::FromTemplate(t) => {
                format!("struct CAN_TMessage_{t}")
            }
        }
    }

    fn struct_def(&self) -> String {
        if let CANMessageKind::FromTemplate(t) = self.kind() {
            return format!(
                "/*  Decoded struct `{}` provided by template `{t}`  */",
                self.struct_ty()
            );
        }

        let mut inner = String::new(); // struct contents

        for sigbit in &self.signals {
            inner += "\n";
            inner += &formatdoc! {"
                /**
                 * -- Signal: {name}
                 *
                 * ----> Description: {desc}
                 * ----> Start bit: {start}
                 * ----> Width: {width}
                 */
                {sigty} {name};
                ",
                name = sigbit.sig.name,
                desc = sigbit.sig.description.as_ref().unwrap_or(&"(None)".into()),
                start = sigbit.start(),
                width = sigbit.sig.width,
                sigty = self.sig_ty_decoded(&sigbit.sig),
            };
        }

        formatdoc! {"
            {} {{
            {}
            }};",
            self.struct_ty(),
            inner.indent(4)
        }
    }

    fn global_struct_ident(&self) -> String {
        format!("CANRX_Message_{}", self.name)
    }

    fn raw_struct_ty(&self) -> String {
        match self.kind() {
            CANMessageKind::Independent => {
                format!("struct CAN_MessageRaw_{}", self.name)
            }
            CANMessageKind::Template => {
                format!("struct CAN_TMessageRaw_{}", self.name)
            }
            CANMessageKind::FromTemplate(t) => {
                format!("struct CAN_TMessageRaw_{t}")
            }
        }
    }

    fn raw_struct_def(&self) -> String {
        if let CANMessageKind::FromTemplate(t) = self.kind() {
            return format!(
                "/*  Raw struct `{}` provided by template `{t}`  */",
                self.raw_struct_ty()
            );
        }

        let mut inner = String::new(); // struct contents

        for sigbit in &self.signals {
            inner += "\n";
            inner += &formatdoc! {"
                /**
                 * -- Raw signal: {name}
                 *
                 * ----> Description: {desc}
                 * ----> Start bit: {start}
                 * ----> Width: {width}
                 */
                {sigty} {name};
                ",
                name = sigbit.sig.name,
                desc = sigbit.sig.description.as_ref().unwrap_or(&"(None)".into()),
                start = sigbit.start(),
                width = sigbit.sig.width,
                sigty = self.sig_ty_raw(&sigbit.sig),
            };
        }

        formatdoc! {"
            {} {{
            {}
            }};",
            self.raw_struct_ty(),
            inner.indent(4)
        }
    }

    fn global_raw_struct_ident(&self) -> String {
        format!("CANRX_MessageRaw_{}", self.name)
    }

    fn rx_fn_name(&self) -> String {
        format!("CANRX_doRx_{}", self.name)
    }

    fn rx_fn_decl(&self) -> String {
        formatdoc! {"
            bool {}(
                const uint8_t * data,
                uint_fast8_t len
            );",
            self.rx_fn_name()
        }
    }

    fn rx_fn_def(&self) -> String {
        /* function comment */
        let comment = formatdoc! {"
            /**
             * Unpacks and decodes message `{}` from raw data.
             *
             * @param data - Input raw data array
             * @param len  - Length of data (must be {} for this function),
             * @param out  - Pointer to output struct
             *
             * @return     - boolean indicating whether decoding was done (len was correct)
             */",
            self.name,
            self.length
        };

        /* arguments */
        let args = formatdoc! {"
            const uint8_t * const data,
            const uint_fast8_t len"
        }
        .indent(4);

        /* length condition check */
        let length_cond = formatdoc! {"
            /*  Check that data length is correct  */
            if (len != {}U) {{
                return false;
            }}",
            self.length
        };

        /* unpacking */
        let unpack_start = formatdoc! {"
            /* ------- Unpack signals ------- */
            {rawty} raw = {{0}};",
            rawty = self.raw_struct_ty()
        };

        let mut unpack = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            let bit = sigbit.start();

            let sig_name = self.normalize_struct_signal_name(&sig.name);

            unpack += &format!(
                "// Unpack `{}`, start bit {}, width {}\n",
                sig.name, bit, sig.width
            );

            // todo: assumes big-endian.
            // step through each of the bit-byte boundaries

            // say the signal is 3 bits wide and starts at position 6.
            // then, select bits from each raw data byte as needed.
            //
            // raw->signal |= ((data[0] & (0b11 << 5)) >> 5) << 0; // select signal bits 0-1 from message bits 6-7 (byte 0 :: bits 6-7)
            // raw->signal |= ((data[1] & (0b1  << 0)) >> 0) << 2; // select signal bit 2 from message bit 8 (byte 1 :: bit 0)
            //                       ^------------------------------- byte
            //                               ^----------------------- mask of bits to select from this byte
            //                                     ^------^---------- ending offset of range within this byte
            //                                                  ^---- current offset within the signal

            assert!(
                sigbit.end() < u8::MAX.into(),
                "Too many bits for me :P, please fix for CAN FD"
            );

            let mut pos = bit;
            let sig_end = sigbit.end();
            while pos <= sig_end {
                let byte = pos / 8;

                let end_of_this_byte = ((byte + 1) * 8) - 1;
                let end_pos = end_of_this_byte.min(sig_end); // either end of this byte or final end of signal
                let end_pos_within_byte = end_pos % 8;

                let num_bits_from_this_byte = end_pos - pos + 1;
                let mask_shift = end_pos_within_byte + 1 - num_bits_from_this_byte;

                let mask: u8 = if num_bits_from_this_byte == 8 {
                    0xFF
                } else {
                    !(!0 << num_bits_from_this_byte)
                };
                let mask = format!("0x{mask:02x}");

                unpack += &formatdoc! {"
                    raw.{name} |= ({rawty})((data[{byte}] & ({mask} << {mask_shift})) >> {mask_shift}) << {sig_pos};\n",
                    name = sig_name,
                    rawty = self.sig_ty_raw(sig),
                    sig_pos = pos - bit
                };

                pos = end_pos + 1;
            }

            unpack += "\n";
        }

        let unpack = unpack.trim();

        /* decode */
        // We need to take each of the raw signals we just unpacked
        // and apply some set of transformations to them.
        //
        // todo for later bounds checks: a facility for signals being strictly enumerated?
        // todo  -> that is, ensure a signal can only be one of its enumerated values

        let decode_start = formatdoc! {"
            /* ------- Decode signals ------- */
            {decty} dec = {{0}};",
            decty = self.struct_ty()
        };

        let mut decode = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            let sig_name = self.normalize_struct_signal_name(&sig.name);

            decode += &formatdoc! {"
                // Decode `{name}`
                dec.{name} = {};

                ",
                self.decoding_expression(sig, &format!("raw.{sig_name}")),
                name = sig_name,
            };
        }

        let decode = decode.trim();

        /* set global variables */
        let set_global = formatdoc! {"
            /* Set global data. */
            {global_raw} = raw;
            {global_dec} = dec;",
            global_raw = self.global_raw_struct_ident(),
            global_dec = self.global_struct_ident(),
        };

        /* stitch it all together */
        let body = formatdoc! {"
            {length_cond}

            {unpack_start}

            {unpack}


            {decode_start}

            {decode}

            {set_global}

            return true;"
        }
        .indent(4);

        formatdoc! {"
            {comment}
            bool {}(\n{args})\n{{
            {body}
            }}",
            self.rx_fn_name(),
        }
    }

    fn tx_fn_name(&self) -> String {
        format!("CANTX_doTx_{}", self.name)
    }

    fn tx_fn_decl(&self) -> String {
        format!(
            "bool {}(uint8_t *data_out, uint8_t *len_out)",
            self.tx_fn_name()
        )
    }

    fn tx_fn_def(&self) -> String {
        /* encoding */
        let mut encode = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            let sig_name = self.normalize_struct_signal_name(&sig.name);

            encode += &formatdoc! {"
                // Encode `{name}`
                raw.{name} = {};

                ",
                self.encoding_expression(sig, &format!("dec.{sig_name}")),
                name = sig_name,
            };
        }

        let encode = encode.trim().indent(4);

        /* packing */

        // similar logic and comments as rx

        // say the signal is 3 bits wide and starts at position 6.
        // then, select bits from the signal to apply to each raw data byte as needed.
        //
        // data[0] |= ((raw->signal & (0b11 << 0)) >> 0) << 5;
        // data[1] |= ((raw->signal & (0b1  << 2)) >> 2) << 0;

        let mut pack = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            let bit = sigbit.start();

            let sig_name = self.normalize_struct_signal_name(&sig.name);

            pack += &format!(
                "// Pack `{}`, start bit {}, width {}\n",
                sig_name, bit, sig.width
            );

            assert!(
                sigbit.end() < u8::MAX.into(),
                "Too many bits for me :P, please fix for CAN FD"
            );

            let mut pos = bit;
            let sig_end = sigbit.end();
            while pos <= sig_end {
                let byte = pos / 8;

                let end_of_this_byte = ((byte + 1) * 8) - 1;
                let end_pos = end_of_this_byte.min(sig_end); // either end of this byte or final end of signal
                let end_pos_within_byte = end_pos % 8;

                let num_bits_from_this_byte = end_pos - pos + 1;
                let mask_shift = end_pos_within_byte + 1 - num_bits_from_this_byte;

                let mask: u8 = if num_bits_from_this_byte == 8 {
                    0xFF
                } else {
                    !(!0 << num_bits_from_this_byte)
                };
                let mask = format!("0x{mask:02x}");

                pack += &formatdoc! {"
                    data_out[{byte}] |= ((raw.{name} & ({mask} << {sig_pos})) >> {sig_pos}) << {mask_shift};\n",
                    name = sig_name,
                    sig_pos = pos - bit
                };

                pos = end_pos + 1;
            }

            pack += "\n";
        }

        let pack = pack.trim().indent(4);

        formatdoc! {"
            // todo: length condition check
            bool {fn_name}(uint8_t * const data_out, uint8_t * const len_out)\n{{
                /* Call user-provided populate function */
                {dec_ty} dec;
                {pop_fn}(&dec); // calls into user code!

                /* ------- Encode signals ------- */
                {raw_ty} raw = {{0}};

            {encode}

                /* ------- Pack signals ------- */

            {pack}

                // Write data length
                *len_out = {length};

                return true;
            }}",
            fn_name = self.tx_fn_name(),
            dec_ty = self.struct_ty(),
            pop_fn = self.tx_populate_fn_name(),
            raw_ty = self.raw_struct_ty(),
            length = self.length,
        }
    }

    fn tx_populate_fn_name(&self) -> String {
        match self.kind() {
            CANMessageKind::Independent => {
                format!("CANTX_populate_{}", self.name)
            }
            CANMessageKind::Template => {
                panic!(
                    "Tried to generate populate function for template `{}`",
                    self.name
                )
            }
            CANMessageKind::FromTemplate(_) => {
                // strip the node name
                // todo: assumes node name is present in the message and is actually the prefix
                let stripped = self
                    .name
                    .strip_prefix(self.tx_node.as_ref().unwrap())
                    .unwrap();
                format!("CANTX_populateTemplate{stripped}")
            }
        }
    }

    fn tx_populate_fn_decl(&self) -> String {
        // include the const in the decl even though we normally wouldn't -
        // user might copy the prototype.
        format!(
            "void {}({} * const m)",
            self.tx_populate_fn_name(),
            self.struct_ty()
        )
    }

    fn getter_fn_decls(&self) -> String {
        let mut getters = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;

            getters += &formatdoc! {"
                {sigty_dec} {fn_name}(void);
                {sigty_raw} {fn_name_raw}(void);

                ",
                sigty_dec = self.sig_ty_decoded(sig),
                sigty_raw = self.sig_ty_raw(sig),
                fn_name = self.getter_fn_name(sig),
                fn_name_raw = self.raw_getter_fn_name(sig),
            }
        }

        getters.trim().into()
    }

    fn getter_fn_defs(&self) -> String {
        let mut getters = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            getters += &formatdoc! {"
                {sigty_dec} {fn_name}(void) {{
                    return {global_decoded}.{name};
                }}

                {sigty_raw} {fn_name_raw}(void) {{
                    return {global_raw}.{name};
                }}

                ",
                name = self.normalize_struct_signal_name(&sig.name),
                sigty_dec = self.sig_ty_decoded(sig),
                sigty_raw = self.sig_ty_raw(sig),
                global_decoded = self.global_struct_ident(),
                global_raw = self.global_raw_struct_ident(),
                fn_name = self.getter_fn_name(sig),
                fn_name_raw = self.raw_getter_fn_name(sig),
            }
        }

        getters.trim().into()
    }

    fn signal_enums(&self) -> String {
        if let CANMessageKind::FromTemplate(t) = self.kind() {
            return format!("/*  Signal enums provied by template `{t}`  */");
        }

        let mut out = String::new();
        let mut some = false;

        for sigbit in &self.signals {
            if let Some(e) = self.c_enum(&sigbit.sig) {
                out += &format!("{e}\n\n");
                some = true;
            }
        }

        if some {
            out.trim().into()
        } else {
            "// (none for this message)".into()
        }
    }

    fn normalize_struct_signal_name(&self, name: &str) -> String {
        if matches!(self.kind(), CANMessageKind::FromTemplate(_)) {
            let prefix = format!("{}_", self.tx_node.as_ref().unwrap());
            name.strip_prefix(&prefix).unwrap().into()
        } else {
            name.into()
        }
    }

    fn tx_stub(&self) -> String {
        formatdoc! {"
            __attribute__((weak)) {} {{
                (void)m;
            }}",
            self.tx_populate_fn_decl()
        }
    }
}
