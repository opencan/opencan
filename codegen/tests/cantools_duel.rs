use std::{ffi::c_int, path::Path, process::Command, collections::HashMap};

use anyhow::{anyhow, Context, Result};
use indoc::formatdoc;
use libloading::{Library, Symbol};
use opencan_core::{CANMessage, CANNetwork, TranslationLayer};
use pyo3::{prelude::*, types::IntoPyDict};
use tempfile::tempdir;

type DecodeFn = unsafe fn(*const u8, u8) -> bool; // todo: u8 is not the right length type - it's uint_fast8_t!

#[test]
fn check_python_env() -> Result<()> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let cantools = py.import("cantools")?;

        let pyver_long: String = sys.getattr("version")?.extract()?;
        let pyver = pyver_long.split_whitespace().next().unwrap();
        let py_semver = semver::Version::parse(pyver)?;
        assert!(semver::VersionReq::parse("3.9")?.matches(&py_semver));

        let cantools_ver: String = cantools.getattr("__version__")?.extract()?;
        let cantools_semver = semver::Version::parse(&cantools_ver)?;
        assert!(semver::VersionReq::parse(">=37.0")?.matches(&cantools_semver));

        Ok(())
    })
}

fn c_to_so(c_file: &Path) -> Result<Library> {
    let temp_dir = tempdir()?;

    let dir = temp_dir.path();
    let so = dir.join("lib.so");

    let c = Command::new("gcc")
        .arg("-Wall")
        .arg("-Wextra")
        // .arg("-Werror")
        .arg("-Wpedantic")
        .arg("-fPIC")
        .arg("-shared")
        .arg(c_file)
        .arg("-o")
        .arg(&so)
        .output()?;

    if !c.status.success() {
        return Err(anyhow!(
            "Failed compiling file {}:\n\n{}",
            c_file.display(),
            String::from_utf8_lossy(&c.stderr)
        ));
    }

    Ok(unsafe { Library::new(&so)? })
}

fn c_string_to_so(content: impl AsRef<[u8]>) -> Result<Library> {
    let temp_dir = tempdir()?;

    let dir = temp_dir.path();
    let c_file = dir.join("c_from_string.c");

    std::fs::write(&c_file, content)?;

    c_to_so(&c_file)
}

#[test]
fn check_cc_works() -> Result<()> {
    let lib = c_string_to_so("int test_sanity(void) { return 99; }\n")?;

    let res = unsafe {
        let check_fn: Symbol<unsafe fn() -> c_int> = lib.get(b"test_sanity")?;

        check_fn()
    };

    assert_eq!(res, 99);
    Ok(())
}

#[test]
fn message_id_lookup() -> Result<()> {
    // Make a CAN network with some messages
    let mut net = CANNetwork::new();
    let node = "TEST";
    net.add_node(node)?;

    let mut make_msg = |name: &str, id: u32| -> Result<CANMessage> {
        let msg = CANMessage::builder()
            .name(format!("{node}_{name}"))
            .id(id)
            .tx_node(node)
            .build()?;

        net.insert_msg(msg.clone())?;

        Ok(msg)
    };

    let msg1 = make_msg("Message1", 0x30)?;
    let msg2 = make_msg("Message2", 0x27)?;

    // Do codegen
    let args = opencan_codegen::Args {
        node: node.into(),
        in_file: "".into(),
    };
    let c_content = opencan_codegen::Codegen::new(args, &net).network_to_c()?;

    // Compile
    println!("{c_content}");
    let lib = c_string_to_so(c_content)?;

    // Look up symbols
    let decode_fn_name = |msg: &CANMessage| format!("CANRX_decode_{}", msg.name);

    let msg1_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg1).as_bytes())? };
    let msg2_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg2).as_bytes())? };
    let lookup: Symbol<fn(u32) -> Option<DecodeFn>> = unsafe { lib.get(b"CANRX_id_to_decode_fn")? };

    assert_eq!(lookup(msg1.id), Some(*msg1_decode));
    assert_eq!(lookup(msg2.id), Some(*msg2_decode));
    assert_eq!(lookup(0x99), None);
    Ok(())
}

#[test]
fn decode_very_basic() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};

    let net = opencan_compose::compose_entry_str(&desc)?;
    let args = opencan_codegen::Args {
        node: "TEST".into(),
        in_file: "".into(),
    };
    let c = opencan_codegen::Codegen::new(args, &net).network_to_c()?;
    let lib = c_string_to_so(c)?;

    let decode: Symbol<DecodeFn> = unsafe { lib.get(b"CANRX_decode_TEST_TestMessage")? };
    let get_raw: Symbol<fn() -> u8> = unsafe { lib.get(b"CANRX_getRaw_TEST_testSignal")? };

    let data: &[u8] = &[0xAF];
    assert_eq!(get_raw(), 0);

    let ret = unsafe { decode(data.as_ptr(), data.len() as u8) };
    assert!(ret);

    assert_eq!(get_raw(), 0xF);

    Ok(())
}

#[test]
fn decode_very_basic_using_cantools() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};

    let net = opencan_compose::compose_entry_str(&desc)?;

    let net_py = opencan_core::CantoolsDecoder::dump_network(&net);

    Python::with_gil(|py| -> Result<()> {
        let locals = [("cantools", py.import("cantools")?)].into_py_dict(py);

        let py_msg = py.eval(&net_py, None, Some(locals))?;

        let data: &[u8] = &[0xAF];
        let sigs_dict = py_msg.call_method1("decode", (data,))?;

        let sig_val: u8 = sigs_dict.get_item("TEST_testSignal")?.extract()?;
        assert_eq!(sig_val, 0xF);

        Ok(())
    })?;

    Ok(())
}

trait Decoder {
    // todo: how to express this?
    // fn new<'a>(net: &'a CANNetwork, node: &str) -> Result<Self>
    //     where Self: Sized;
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, u8)>>;
}

struct CodegenDecoder<'n> {
    net: &'n CANNetwork,
    lib: Library,
}

impl<'n> CodegenDecoder<'n> {
    fn new(net: &'n CANNetwork, node: &str) -> Result<CodegenDecoder<'n>> {
        let args = opencan_codegen::Args {
            node: node.into(),
            in_file: "".into(),
        };

        let c = opencan_codegen::Codegen::new(args, net).network_to_c()?;
        let lib = c_string_to_so(c)?;

        Ok(Self { net, lib })
    }
}

impl Decoder for CodegenDecoder<'_> {
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, u8)>> {
        let decode_fn_name = format!("CANRX_decode_{msg}");
        let decode: Symbol<DecodeFn> = unsafe { self.lib.get(decode_fn_name.as_bytes())? };

        let ret = unsafe { decode(data.as_ptr(), data.len() as u8) };
        if !ret {
            return Err(anyhow!(
                "Generated decode function failed to decode `{msg}`."
            ));
        }

        let mut sigvals = vec![];

        for sigbit in &self
            .net
            .message_by_name(msg)
            .context("Message doesn't exist")?
            .signals
        {
            let raw_fn_name = format!("CANRX_getRaw_{}", sigbit.sig.name);
            let raw_fn: Symbol<fn() -> u8> = unsafe { self.lib.get(raw_fn_name.as_bytes())? };

            sigvals.push((sigbit.sig.name.clone(), raw_fn()));
        }

        sigvals.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));

        Ok(sigvals)
    }
}

#[test]
fn test_decode_with_trait() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};
    let net = opencan_compose::compose_entry_str(&desc)?;
    let decoder = CodegenDecoder::new(&net, "TEST")?;

    let v = decoder.decode_message("TEST_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TEST_testSignal");
    assert_eq!(*val, 0xA);

    Ok(())
}

struct CantoolsDecoder<'n> {
    net: &'n CANNetwork,
    node: String,
}

impl<'n> CantoolsDecoder<'n> {
    fn new(net: &'n CANNetwork, node: &str) -> Result<CantoolsDecoder<'n>> {
        Ok(Self {
            net,
            node: node.into(),
        })
    }
}

impl Decoder for CantoolsDecoder<'_> {
    fn decode_message(&self, msg: &str, data: &[u8]) -> Result<Vec<(String, u8)>> {
        // pretty much stateless.

        Python::with_gil(|py| -> Result<_> {
            // import cantools
            let locals = [("cantools", py.import("cantools")?)].into_py_dict(py);

            // translate message to Python object
            let net_msg = self.net.message_by_name(msg).context("Message doesn't exist")?;

            let py_msg_code = opencan_core::CantoolsDecoder::dump_message(net_msg);
            let py_msg = py.eval(&py_msg_code, None, Some(locals))?;

            // decode signals
            let sigs_dict = py_msg.call_method1("decode", (data,))?;

            let sigs_map: HashMap<String, u8> = sigs_dict.extract()?;

            let mut sigs_vec: Vec<(String, u8)> = sigs_map.into_iter().collect();
            sigs_vec.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));

            Ok(sigs_vec)
        })
    }
}

#[test]
fn test_decode_with_trait_cantools() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};
    let net = opencan_compose::compose_entry_str(&desc)?;
    let decoder = CantoolsDecoder::new(&net, "TEST")?;

    let v = decoder.decode_message("TEST_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TEST_testSignal");
    assert_eq!(*val, 0xA);

    Ok(())
}
