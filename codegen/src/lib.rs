use std::fmt::Display;

use anyhow::{anyhow, Result};
use clap::Parser;
use opencan_core::{CANNetwork, CANSignal};

#[derive(Parser)]
pub struct Args {
    pub node: String,
    pub in_file: String,
}

pub struct Codegen {}

pub enum CSignalTy {
    U8,
    U16,
    U32,
    U64,
    Float,
}

impl Display for CSignalTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::U8 => "uint8_t",
                Self::U16 => "uint16_t",
                Self::U32 => "uint32_t",
                Self::U64 => "uint64_t",
                Self::Float => "float", // todo: use a typedef?
            }
        )
    }
}

impl Codegen {
    pub fn network_to_c(args: Args, net: CANNetwork) -> Result<String> {
        let mut output = String::new();

        let node_msgs = net
            .messages_by_node(&args.node)
            .ok_or(anyhow!("Node `{}` not found in network.", args.node))?;

        for msg in node_msgs {
            // generate structs
            // ok, for this message, let's generate a struct for each signal
            output += &format!("struct CAN_Message_{} {{", msg.name);

            for sigbit in &msg.signals {
                output += "\n";

                /* start comment block with signal name */
                output += &format!("    /* --- Signal: {}\n", sigbit.sig.name);

                /* description */
                if let Some(d) = &sigbit.sig.description {
                    output += "     *\n";
                    output += &format!("     * ----> Description: \"{d}\"\n");
                }

                /* start bit */
                output += &format!("     * ----> Start bit: {}\n", sigbit.bit);

                /* finish comment block */
                output += "     */\n";

                output += &format!(
                    "    {} {};\n",
                    Self::get_ty_for_decoded_signal(&sigbit.sig),
                    sigbit.sig.name
                );
            }

            output += "};\n\n";
        }

        Ok(output)
    }

    /// Get the C type for the decoded signal, not taking into account minimum/maximum capping.
    /// This is always float right now.
    fn get_ty_for_decoded_signal(_sig: &CANSignal) -> CSignalTy {
        // todo: need integer signal bounds support
        // should we make this implicit or explicit... hmmm...
        // making it implicit (i.e. say 1 instead of 1.0) might be obtuse / ambiguous
        // -> otoh, saying force_integer: yes or force_float: yes all the time is annnoying

        // I think I lean implicit. The problem is then it becomes a nightmare in Rust code....

        CSignalTy::Float
    }

    fn _get_ty_for_raw_signal(sig: &CANSignal) -> CSignalTy {
        // this is totally wrong, what a fail
        match sig.width {
            1..=8 => CSignalTy::U8,
            ..=16 => CSignalTy::U16,
            ..=32 => CSignalTy::U32,
            ..=64 => CSignalTy::U64,
            w => panic!(
                "Unexpectedly wide signal: `{}` is `{}` bits wide",
                sig.name, w
            ),
        }
    }
}
