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
            output += &format!("struct CAN_Message_{} {{\n", msg.name);

            for sigbit in &msg.signals {
                output += &format!(
                    "    // --- Signal: {}, offset: {}\n",
                    sigbit.sig.name, sigbit.bit
                );

                output += &format!(
                    "    {} {};\n",
                    Self::get_ty_for_signal(&sigbit.sig), sigbit.sig.name);

                output += "\n";
            }

            output += "};\n\n";
        }

        Ok(output)
    }

    fn get_ty_for_signal(sig: &CANSignal) -> CSignalTy {
        // this is totally wrong, what a fail
        match sig.width {
            1..=8 => CSignalTy::U8,
            ..=16 => CSignalTy::U16,
            ..=32 => CSignalTy::U32,
            ..=64 => CSignalTy::U64,
            w => panic!("Unexpectedly wide signal: `{}` is `{}` bits wide", sig.name, w)
        }
    }
}
