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

struct CodegenCtxt {
    content: String,
    indent: usize,
}

impl ToString for CodegenCtxt {
    fn to_string(&self) -> String {
        self.content.clone()
    }
}

impl CodegenCtxt {
    fn get_prefix(&self) -> String {
        " ".repeat(4 * self.indent)
    }

    fn push(&mut self, next: Self) {
        self.content += &textwrap::indent(&next.content, &next.get_prefix());
    }

    fn new_indented(&self) -> Self {
        return Self {
            content: String::new(),
            indent: self.indent + 1,
        };
    }

    fn new() -> Self {
        return Self {
            content: String::new(),
            indent: 0,
        };
    }

    fn str(&mut self, s: impl AsRef<str>) {
        self.content += s.as_ref();
    }

    fn line(&mut self, s: impl AsRef<str>) {
        self.str(format!("{}\n", s.as_ref()));
    }

    fn newline(&mut self) {
        self.content += "\n";
    }
}

impl Codegen {
    pub fn network_to_c(args: Args, net: CANNetwork) -> Result<String> {
        let mut output = CodegenCtxt::new();

        let node_msgs = net
            .messages_by_node(&args.node)
            .ok_or(anyhow!("Node `{}` not found in network.", args.node))?;

        for msg in node_msgs {
            // generate structs
            // ok, for this message, let's generate a struct for each signal
            output.str(format!("struct CAN_Message_{} {{", msg.name));

            let mut s = output.new_indented();

            for sigbit in &msg.signals {
                s.newline();

                /* start comment block with signal name */
                s.line(format!("/* --- Signal: {}", sigbit.sig.name));

                /* description */
                if let Some(d) = &sigbit.sig.description {
                    s.line(" *");
                    s.line(format!(" * ----> Description: \"{d}\""));
                }

                /* start bit */
                s.line(format!(" * ----> Start bit: {}", sigbit.bit));

                /* finish comment block */
                s.line(" */");

                s.line(format!(
                    "{} {};",
                    Self::get_ty_for_decoded_signal(&sigbit.sig),
                    sigbit.sig.name
                ));
            }
            output.push(s);
            output.line("};");
            output.newline();
        }

        Ok(output.to_string())
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
