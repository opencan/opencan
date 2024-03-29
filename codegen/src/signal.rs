use std::fmt::Display;

use indoc::formatdoc;
use opencan_core::{CANMessage, CANMessageKind, CANSignal};

use crate::{message::MessageCodegen, Indent};

pub enum CSignalTy {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Float,
    Enum(String),
}

impl Display for CSignalTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bool => "bool",
                Self::U8 => "uint8_t",
                Self::I8 => "int8_t",
                Self::U16 => "uint16_t",
                Self::I16 => "int16_t",
                Self::U32 => "uint32_t",
                Self::I32 => "int32_t",
                Self::U64 => "uint64_t",
                Self::I64 => "int64_t",
                Self::Float => "float", // todo: use a typedef?
                Self::Enum(s) => s,
            }
        )
    }
}

pub trait SignalCodegen {
    /// C type for this signal for unpacking.
    ///
    /// This is like the raw type, but always unsigned,
    /// as it is pre-sign extension.
    fn sig_ty_raw_before_sign_extension(&self, sig: &CANSignal) -> CSignalTy;
    /// C type for this signal's raw value.
    fn sig_ty_raw(&self, sig: &CANSignal) -> CSignalTy;
    /// Get the C type for the decoded signal.
    ///
    /// This does not take into account minimum/maximum capping - that is, this
    /// gives the type for the entire _representable_ decoded range, not just
    /// what's within the minimum/maximum additional bounds.
    fn sig_ty_decoded(&self, sig: &CANSignal) -> CSignalTy;
    /// Whether this signal needs sign extension.
    ///
    /// Signals that are twos-complement and whose width is not an even power
    /// of two need sign extension.
    fn sig_needs_sign_extension(&self, sig: &CANSignal) -> bool;

    /// C enumeration for this signal's enumerated values, if any.
    fn c_enum(&self, sig: &CANSignal) -> Option<String>;

    /// Name of the C getter function for this signal's decoded value.
    fn getter_fn_name(&self, sig: &CANSignal) -> String;
    /// Name of the C getter function for this signal's raw value.
    fn raw_getter_fn_name(&self, sig: &CANSignal) -> String;

    /// Conversion expression from raw signal to decoded signal.
    fn decoding_expression(&self, sig: &CANSignal, raw_rvalue: &str) -> String;
    /// Conversion expression from decoded signal to raw signal.
    fn encoding_expression(&self, sig: &CANSignal, dec_rvalue: &str) -> String;
}

impl SignalCodegen for CANMessage {
    fn sig_ty_raw_before_sign_extension(&self, sig: &CANSignal) -> CSignalTy {
        match self.sig_ty_raw(sig) {
            CSignalTy::Bool => CSignalTy::Bool,
            CSignalTy::I8 | CSignalTy::U8 => CSignalTy::U8,
            CSignalTy::I16 | CSignalTy::U16 => CSignalTy::U16,
            CSignalTy::I32 | CSignalTy::U32 => CSignalTy::U32,
            CSignalTy::I64 | CSignalTy::U64 => CSignalTy::U64,
            t => panic!("Unexpected raw type {t} for signal {}", sig.name),
        }
    }

    fn sig_ty_raw(&self, sig: &CANSignal) -> CSignalTy {
        if sig.twos_complement {
            match sig.width {
                1 => panic!(
                    "Signal `{}` has width 1 but also twos_complement=true",
                    sig.name
                ),
                2..=8 => CSignalTy::I8,
                9..=16 => CSignalTy::I16,
                17..=32 => CSignalTy::I32,
                33..=64 => CSignalTy::I64,
                w => panic!(
                    "Unexpectedly wide signal: `{}` is `{}` bits wide",
                    sig.name, w
                ),
            }
        } else {
            match sig.width {
                1 => CSignalTy::Bool,
                2..=8 => CSignalTy::U8,
                9..=16 => CSignalTy::U16,
                17..=32 => CSignalTy::U32,
                33..=64 => CSignalTy::U64,
                w => panic!(
                    "Unexpectedly wide signal: `{}` is `{}` bits wide",
                    sig.name, w
                ),
            }
        }
    }

    fn sig_needs_sign_extension(&self, sig: &CANSignal) -> bool {
        matches!(
            self.sig_ty_raw(sig),
            CSignalTy::I8 | CSignalTy::I16 | CSignalTy::I32 | CSignalTy::I64
        ) && !sig.width.is_power_of_two()
    }

    fn sig_ty_decoded(&self, sig: &CANSignal) -> CSignalTy {
        // todo: support for both enumerated and continuous decoded getters
        if !sig.enumerated_values.is_empty() {
            // CSignalTy::Enum(format!("enum CAN_{}", sig.name))
            let name = match self.kind() {
                CANMessageKind::Raw => panic!("Raw message should not have signals"),
                CANMessageKind::Independent => format!("enum CAN_{}", sig.name),
                CANMessageKind::Template => format!("enum CAN_T_{}_{}", self.name, sig.name),
                CANMessageKind::FromTemplate(t) => format!(
                    "enum CAN_T_{t}_{}",
                    self.normalize_struct_signal_name(&sig.name)
                ),
            };

            CSignalTy::Enum(name)
        } else if sig.scale.is_none() && sig.offset.is_none() {
            //
            // todo: complete integer signal bounds support
            // should we make this implicit or explicit... hmmm...
            // making it implicit (i.e. say 1 instead of 1.0) might be obtuse / ambiguous
            // -> otoh, saying force_integer: yes or force_float: yes all the time is annnoying

            // I think I lean implicit. The problem is then it becomes a nightmare in Rust code....

            // for now, if the signal has no offset or scale, then return its raw type, else float.
            //
            self.sig_ty_raw(sig)
        } else {
            CSignalTy::Float
        }
    }

    fn c_enum(&self, sig: &CANSignal) -> Option<String> {
        let CSignalTy::Enum(ty) = self.sig_ty_decoded(sig) else {
            return None; // decoded type is not an enum
        };

        // sort enumerated values since they're in random order in the map
        let mut evs: Vec<_> = sig.enumerated_values.iter().collect();
        evs.sort_by_key(|ev| ev.1);

        // choose prefix
        let prefix = match self.kind() {
            CANMessageKind::Raw => panic!("Raw message should not have enums"),
            CANMessageKind::Independent => "CAN".into(),
            CANMessageKind::Template => format!("CAN_T_{}", self.name.to_uppercase()),
            CANMessageKind::FromTemplate(t) => format!("CAN_T_{}", t.to_uppercase()),
        };

        // collect C enum values
        let mut inner = String::new();
        for e in evs {
            inner += &format!(
                "{prefix}_{}_{} = {},\n",
                self.normalize_struct_signal_name(&sig.name).to_uppercase(),
                e.0,
                e.1
            );
        }

        Some(formatdoc! {"
            {} {{
            {}
            }};",
            ty,
            inner.trim().indent(4)
        })
    }

    fn getter_fn_name(&self, sig: &CANSignal) -> String {
        format!("CANRX_get_{}", sig.name)
    }

    fn raw_getter_fn_name(&self, sig: &CANSignal) -> String {
        format!("CANRX_getRaw_{}", sig.name)
    }

    fn decoding_expression(&self, sig: &CANSignal, raw_rvalue: &str) -> String {
        // Currently, signals are either their raw type if they have no scale
        // or offset, or they're CSignalTy::Float if they have a scale or offset.
        //
        // We're not accounting for enumerated values yet, which we may or may not
        // do at all in this function.

        if matches!(self.sig_ty_decoded(sig), CSignalTy::Float) {
            let scale = sig.scale.map_or("".into(), |s| format!(" * {s:?}f"));
            let offset = sig.offset.map_or("".into(), |o| format!(" + {o:?}f"));

            format!(
                "(({float_ty})({raw_rvalue}){scale}){offset}",
                float_ty = CSignalTy::Float
            )
        } else {
            // Just copy the raw signal.

            // For now, these should be None according to the logic in .c_ty_decoded()
            assert!(sig.offset.is_none());
            assert!(sig.scale.is_none());

            raw_rvalue.into()
        }
    }

    // Similar logic and notes as above
    fn encoding_expression(&self, sig: &CANSignal, dec_rvalue: &str) -> String {
        if matches!(self.sig_ty_decoded(sig), CSignalTy::Float) {
            let scale = sig.scale.map_or("".into(), |s| format!(" / {s:?}f"));
            let offset = sig.offset.map_or("".into(), |o| format!(" - {o:?}f"));

            format!(
                "({raw_ty})((({dec_rvalue}){scale}){offset})",
                raw_ty = self.sig_ty_raw(sig)
            )
        } else {
            assert!(sig.offset.is_none());
            assert!(sig.scale.is_none());

            dec_rvalue.into()
        }
    }
}
