use std::{ffi::c_int, path::Path, process::Command};

use anyhow::{anyhow, Result};
use libloading::{Library, Symbol};
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
        .arg("-Werror")
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

#[test]
fn check_cc_works() -> Result<()> {
    let temp_dir = TempDir::new("check_cc_works")?;

    let dir = temp_dir.path();
    let c_file = dir.join("check.c");

    std::fs::write(&c_file, "int test_sanity(void) { return 99; }\n")?;

    let lib = c_to_so(&c_file)?;

    let res = unsafe {
        let check_fn: Symbol<unsafe fn() -> c_int> = lib.get(b"test_sanity")?;

        check_fn()
    };

    assert_eq!(res, 99);
    Ok(())
}
