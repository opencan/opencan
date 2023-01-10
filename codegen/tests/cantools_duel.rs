use std::{ffi::c_int, path::Path, process::Command};

use anyhow::{anyhow, Result};
use libloading::{Library, Symbol};
use opencan_core::{CANMessage, CANNetwork};
use pyo3::prelude::*;
use tempdir::TempDir;

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
        assert!(semver::VersionReq::parse("37.0")?.matches(&cantools_semver));

        Ok(())
    })
}

fn c_to_so(c_file: &Path) -> Result<Library> {
    let temp_dir = TempDir::new("c_to_so")?;

    let dir = temp_dir.path();
    let so = dir.join("lib.so");

    let c = Command::new("gcc")
        .arg("-Wall")
        .arg("-Wextra")
        // .arg("-Werror")
        .arg("-Wpedantic")
        .arg("-shared")
        .arg(&c_file)
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
    let temp_dir = TempDir::new("c_string_to_so")?;

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
fn test_message_id_lookup() -> Result<()> {
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
    let c_content = opencan_codegen::Codegen::new(args, net).network_to_c()?;

    // Compile
    println!("{c_content}");
    let lib = c_string_to_so(c_content)?;

    // Look up symbols
    type DecodeFn = unsafe fn();

    let decode_fn_name = |msg: &CANMessage| format!("CANRX_decode_{}", msg.name);

    let msg1_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg1).as_bytes())? };
    let msg2_decode: Symbol<DecodeFn> = unsafe { lib.get(decode_fn_name(&msg2).as_bytes())? };
    let lookup: Symbol<fn(u32) -> Option<DecodeFn>> = unsafe { lib.get(b"CANRX_id_to_decode_fn")? };

    assert_eq!(lookup(msg1.id), Some(*msg1_decode));
    assert_eq!(lookup(msg2.id), Some(*msg2_decode));
    assert_eq!(lookup(0x99), None);
    Ok(())
}
