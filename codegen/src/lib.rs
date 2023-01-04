use std::fmt::Display;

use anyhow::{anyhow, Result};
use clap::Parser;
use indoc::formatdoc;
use opencan_core::{CANMessage, CANNetwork, CANSignal};

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

struct CodeCtxt {
    content: String,
}

impl ToString for CodeCtxt {
    fn to_string(&self) -> String {
        self.content.clone()
    }
}

impl CodeCtxt {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, next: Self, indent: usize) {
        let prefix = " ".repeat(4 * indent);
        self.content += &textwrap::indent(&next.content, &prefix);
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
        let mut output = CodeCtxt::new();

        let node_msgs = net
            .messages_by_node(&args.node)
            .ok_or(anyhow!("Node `{}` not found in network.", args.node))?;

        for msg in node_msgs {
            output.push(Self::struct_for_message(msg), 0);
            output.push(Self::decode_fn_for_message(msg), 0);
        }

        Ok(output.to_string())
    }

    fn decode_fn_for_message(msg: &CANMessage) -> CodeCtxt {
        let mut top = CodeCtxt::new();

        top.line(formatdoc!(
            "
            {} {}(const uint64_t data) {{

            }}
            ",
            Self::struct_name_for_message(msg),
            Self::decode_fn_name_for_message(msg)
        ));

        top
    }

    fn struct_for_message(msg: &CANMessage) -> CodeCtxt {
        // generate structs
        // ok, for this message, let's generate a struct for each signal
        let mut top = CodeCtxt::new();
        let mut inner = CodeCtxt::new(); // struct contents

        top.str(format!("{} {{", Self::struct_name_for_message(msg)));

        for sigbit in &msg.signals {
            inner.newline();

            inner.line(formatdoc!(
                "
                /**
                 * -- Signal: {}
                 *
                 * ----> Description: {}
                 * ----> Start bit: {}
                 */
                {} {};
                ",
                &sigbit.sig.name,
                sigbit.sig.description.as_ref().unwrap_or(&"(None)".into()),
                sigbit.bit,
                Self::ty_for_decoded_signal(&sigbit.sig),
                sigbit.sig.name
            ));
        }

        top.push(inner, 1);
        top.line("};");
        top.newline();

        top
    }

    fn struct_name_for_message(msg: &CANMessage) -> String {
        format!("struct CAN_Message_{}", msg.name)
    }

    fn decode_fn_name_for_message(msg: &CANMessage) -> String {
        format!("CANRX_decode_{}", msg.name)
    }

    /// Get the C type for the decoded signal.
    ///
    /// This does not take into account minimum/maximum capping - that is, this
    /// gives the type for the entire _representable_ decoded range, not just
    /// what's within the minimum/maximum additional bounds.
    fn ty_for_decoded_signal(sig: &CANSignal) -> CSignalTy {
        // todo: complete integer signal bounds support
        // should we make this implicit or explicit... hmmm...
        // making it implicit (i.e. say 1 instead of 1.0) might be obtuse / ambiguous
        // -> otoh, saying force_integer: yes or force_float: yes all the time is annnoying

        // I think I lean implicit. The problem is then it becomes a nightmare in Rust code....

        // for now, if the signal has no offset or scale, then return its raw type, else float.
        if sig.scale.is_none() && sig.offset.is_none() {
            Self::ty_for_raw_signal(sig)
        } else {
            CSignalTy::Float
        }
    }

    fn ty_for_raw_signal(sig: &CANSignal) -> CSignalTy {
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
