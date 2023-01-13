use indoc::formatdoc;
use opencan_core::CANMessage;

use crate::{signal::*, Indent};

pub trait MessageCodegen {
    fn struct_ty(&self) -> String;
    fn struct_def(&self) -> String;
    fn global_struct_ident(&self) -> String;

    fn raw_struct_ty(&self) -> String;
    fn raw_struct_def(&self) -> String;
    fn global_raw_struct_ident(&self) -> String;

    fn decode_fn_name(&self) -> String;
    fn decode_fn_def(&self) -> String;

    fn getter_fn_defs(&self) -> String;
}

impl MessageCodegen for CANMessage {
    fn struct_ty(&self) -> String {
        format!("struct CAN_Message_{}", self.name)
    }

    fn struct_def(&self) -> String {
        let mut top = String::new();
        let mut inner = String::new(); // struct contents

        top += &format!("{} {{", self.struct_ty());

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
                start = sigbit.bit,
                width = sigbit.sig.width,
                sigty = sigbit.sig.c_ty_decoded(),
            };
        }

        top += &inner.indent(4);
        top += "};";

        top
    }

    fn global_struct_ident(&self) -> String {
        format!("CANRX_Message_{}", self.name)
    }

    fn raw_struct_ty(&self) -> String {
        format!("struct CAN_MessageRaw_{}", self.name)
    }

    fn raw_struct_def(&self) -> String {
        let mut top = String::new();
        let mut inner = String::new(); // struct contents

        top += &format!("{} {{", self.raw_struct_ty());

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
                start = sigbit.bit,
                width = sigbit.sig.width,
                sigty = sigbit.sig.c_ty_raw(),
            };
        }

        top += &inner.indent(4);
        top += "};";

        top
    }

    fn global_raw_struct_ident(&self) -> String {
        format!("CANRX_MessageRaw_{}", self.name)
    }

    fn decode_fn_name(&self) -> String {
        format!("CANRX_decode_{}", self.name)
    }

    fn decode_fn_def(&self) -> String {
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
            /*  Unpack signals  */
            {rawty} raw = {{0}};",
            rawty = self.raw_struct_ty()
        };

        let mut unpack = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            let bit = sigbit.bit;

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
                    name = sig.name,
                    rawty = sig.c_ty_raw(),
                    sig_pos = pos - bit
                };

                pos = end_pos + 1;
            }

            unpack += "\n";
        }

        unpack = unpack.trim().into();

        /* set global variables */
        let set_global = formatdoc! {"
            /* Set global data. */
            {global_raw} = raw;",
            global_raw = self.global_raw_struct_ident()
        };

        /* stitch it all together */
        let body = formatdoc! {"
            {length_cond}

            {unpack_start}

            {unpack}

            {set_global}

            return true;"
        }
        .indent(4);

        formatdoc! {"
            {comment}
            bool {}(\n{args})\n{{
            {body}
            }}",
            self.decode_fn_name(),
        }
    }

    fn getter_fn_defs(&self) -> String {
        let mut getters = String::new();

        for sigbit in &self.signals {
            let sig = &sigbit.sig;
            getters += &formatdoc! {"\n
                {sigty_dec} {fn_name}(void) {{
                    return {global_decoded}.{name};
                }}

                {sigty_raw} {fn_name_raw}(void) {{
                    return {global_raw}.{name};
                }}",
                name = sig.name,
                sigty_dec = sig.c_ty_decoded(),
                sigty_raw = sig.c_ty_raw(),
                global_decoded = self.global_struct_ident(),
                global_raw = self.global_raw_struct_ident(),
                fn_name = sig.getter_fn_name(),
                fn_name_raw = sig.raw_getter_fn_name(),
            }
        }

        getters.trim().into()
    }
}
