use std::{collections::HashMap, ffi::c_float};

use anyhow::{anyhow, Context, Result};
use libloading::{Library, Symbol};
use opencan_codegen::signal::{CSignalTy as CodegenCSignalTy, SignalCodegen};
use opencan_core::{translation::CantoolsTranslator, CANNetwork, Translation};
use pyo3::{prelude::*, types::IntoPyDict};

use crate::util::*;

pub type DecodeFn = unsafe fn(*const u8, u8) -> bool; // todo: u8 is not the right length type - it's uint_fast8_t!

#[derive(Debug, PartialEq)]
pub enum SignalValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Float(c_float),
}

pub trait Decoder {
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, SignalValue)>>;
}

pub struct CodegenDecoder<'n> {
    pub net: &'n CANNetwork,
    pub lib: Library,
}

impl<'n> CodegenDecoder<'n> {
    pub fn new(net: &'n CANNetwork, node: &str) -> Result<CodegenDecoder<'n>> {
        let args = opencan_codegen::Args {
            node: node.into(),
            tx_stubs: true,
            rx_callback_stubs: true,
        };

        let stubs = include_str!("test_callback_stubs.c");

        let c = opencan_codegen::Codegen::new(args, net)?.network_to_c();
        let lib = c_strings_to_so([c.as_list(), vec![("test_callback_stubs.c", stubs)]].concat())?;

        Ok(Self { net, lib })
    }
}

impl Decoder for CodegenDecoder<'_> {
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, SignalValue)>> {
        let decode_fn_name = format!("CANRX_doRx_{msg}");
        let decode: Symbol<DecodeFn> = unsafe { self.lib.get(decode_fn_name.as_bytes())? };

        let ret = unsafe { decode(data.as_ptr(), data.len() as u8) };
        if !ret {
            return Err(anyhow!(
                "Generated decode function failed to decode `{msg}`."
            ));
        }

        let mut sigvals = vec![];

        let msg = self
            .net
            .message_by_name(msg)
            .context(format!("Message `{msg}` doesn't exist"))?;

        for sigbit in &msg.signals {
            let raw_fn_name = format!("CANRX_getRaw_{}", sigbit.sig.name);
            let raw_fn_name = raw_fn_name.as_bytes();

            let val = match msg.sig_ty_raw(&sigbit.sig) {
                CodegenCSignalTy::Bool => {
                    let raw_fn: Symbol<fn() -> bool> = unsafe { self.lib.get(raw_fn_name)? };
                    SignalValue::Bool(raw_fn())
                }
                CodegenCSignalTy::U8 => {
                    let raw_fn: Symbol<fn() -> u8> = unsafe { self.lib.get(raw_fn_name)? };
                    SignalValue::U8(raw_fn())
                }
                CodegenCSignalTy::U16 => {
                    let raw_fn: Symbol<fn() -> u16> = unsafe { self.lib.get(raw_fn_name)? };
                    SignalValue::U16(raw_fn())
                }
                CodegenCSignalTy::U32 => {
                    let raw_fn: Symbol<fn() -> u32> = unsafe { self.lib.get(raw_fn_name)? };
                    SignalValue::U32(raw_fn())
                }
                CodegenCSignalTy::U64 => {
                    let raw_fn: Symbol<fn() -> u64> = unsafe { self.lib.get(raw_fn_name)? };
                    SignalValue::U64(raw_fn())
                }
                t => panic!("Unexpected signal type `{t}` for raw codegen decode"),
            };

            sigvals.push((sigbit.sig.name.clone(), val));
        }

        sigvals.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));

        Ok(sigvals)
    }
}

pub struct CantoolsDecoder<'n> {
    net: &'n CANNetwork,
}

impl<'n> CantoolsDecoder<'n> {
    pub fn new(net: &'n CANNetwork) -> Result<CantoolsDecoder<'n>> {
        Ok(Self { net })
    }
}

impl Decoder for CantoolsDecoder<'_> {
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, SignalValue)>> {
        // pretty much stateless.

        Python::with_gil(|py| -> Result<_> {
            // import cantools
            let locals = [("cantools", py.import("cantools")?)].into_py_dict(py);

            // translate message to Python object
            let net_msg = self
                .net
                .message_by_name(msg)
                .context(format!("Message `{msg}` doesn't exist"))?;

            let py_msg_code = CantoolsTranslator::dump_message(net_msg);
            let py_msg = py.eval(&py_msg_code, None, Some(locals))?;

            // decode signals
            let sigs_dict = py_msg.call_method1("decode", (data, false, false))?;

            let sigs_map: HashMap<String, &PyAny> = sigs_dict.extract()?;

            let mut sigvals = vec![];

            for sigbit in &net_msg.signals {
                let val = match net_msg.sig_ty_raw(&sigbit.sig) {
                    CodegenCSignalTy::Bool => {
                        // extract as u8 and then convert to bool with `!= 0`, otherwise TypeError from pyo3
                        SignalValue::Bool(
                            sigs_map.get(&sigbit.sig.name).unwrap().extract::<u8>()? != 0,
                        )
                    }
                    CodegenCSignalTy::U8 => {
                        SignalValue::U8(sigs_map.get(&sigbit.sig.name).unwrap().extract()?)
                    }
                    CodegenCSignalTy::U16 => {
                        SignalValue::U16(sigs_map.get(&sigbit.sig.name).unwrap().extract()?)
                    }
                    CodegenCSignalTy::U32 => {
                        SignalValue::U32(sigs_map.get(&sigbit.sig.name).unwrap().extract()?)
                    }
                    CodegenCSignalTy::U64 => {
                        SignalValue::U64(sigs_map.get(&sigbit.sig.name).unwrap().extract()?)
                    }
                    t => panic!("Unexpected signal type `{t}` for raw cantools decode"),
                };

                sigvals.push((sigbit.sig.name.clone(), val));
            }

            sigvals.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));

            Ok(sigvals)
        })
    }
}
