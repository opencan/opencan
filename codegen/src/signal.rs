use std::fmt::Display;

use opencan_core::CANSignal;

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

pub trait SignalCodegen {
    /// C type for this signal's raw value.
    fn c_ty_raw(&self) -> CSignalTy;
    /// C type for this signal's decoded value.
    fn c_ty_decoded(&self) -> CSignalTy;

    /// Name of the C getter function for this signal's decoded value.
    fn getter_fn_name(&self) -> String;
    /// Name of the C getter function for this signal's raw value.
    fn raw_getter_fn_name(&self) -> String;

    /// Conversion expression from raw signal to decoded signal.
    fn decoding_expression(&self, raw_rvalue: &str) -> String;
}

impl SignalCodegen for CANSignal {
    fn c_ty_raw(&self) -> CSignalTy {
        match self.width {
            1..=8 => CSignalTy::U8,
            9..=16 => CSignalTy::U16,
            17..=32 => CSignalTy::U32,
            33..=64 => CSignalTy::U64,
            w => panic!(
                "Unexpectedly wide signal: `{}` is `{}` bits wide",
                self.name, w
            ),
        }
    }

    /// Get the C type for the decoded signal.
    ///
    /// This does not take into account minimum/maximum capping - that is, this
    /// gives the type for the entire _representable_ decoded range, not just
    /// what's within the minimum/maximum additional bounds.
    fn c_ty_decoded(&self) -> CSignalTy {
        // todo: complete integer signal bounds support
        // should we make this implicit or explicit... hmmm...
        // making it implicit (i.e. say 1 instead of 1.0) might be obtuse / ambiguous
        // -> otoh, saying force_integer: yes or force_float: yes all the time is annnoying

        // I think I lean implicit. The problem is then it becomes a nightmare in Rust code....

        // for now, if the signal has no offset or scale, then return its raw type, else float.
        if self.scale.is_none() && self.offset.is_none() {
            self.c_ty_raw()
        } else {
            CSignalTy::Float
        }
    }

    fn getter_fn_name(&self) -> String {
        format!("CANRX_get_{}", self.name)
    }

    fn raw_getter_fn_name(&self) -> String {
        format!("CANRX_getRaw_{}", self.name)
    }

    fn decoding_expression(&self, raw_rvalue: &str) -> String {
        // Currently, signals are either their raw type if they have no scale
        // or offset, or they're CSignalTy::Float if they have a scale or offset.
        //
        // We're not accounting for enumerated values yet, which we may or may not
        // do at all in this function.

        if let CSignalTy::Float = self.c_ty_decoded() {
            let scale = self.scale.map_or("".into(), |s| format!(" * {s}f"));
            let offset = self.offset.map_or("".into(), |o| format!(" + {o}f"));

            format!(
                "(({float_ty})({raw_rvalue}){scale}){offset}",
                float_ty = CSignalTy::Float
            )
        } else {
            // Just copy the raw signal.

            // For now,, these should be None according to the logic in .c_ty_decoded()
            assert!(self.offset.is_none());
            assert!(self.scale.is_none());

            raw_rvalue.into()
        }
    }
}
