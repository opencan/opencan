use std::{ffi::c_int, process::Command};

use anyhow::Result;
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

#[test]
fn check_cc_works() -> Result<()> {
    let dir = TempDir::new("check_cc_works")?;
    let c_file = dir.path().join("check.c");
    let so = dir.path().join("check.so");

    std::fs::write(&c_file, r#"int test_sanity(void) { return 99; } "#)?;

    let c = Command::new("gcc")
        .arg("-shared")
        .arg(c_file)
        .arg("-o")
        .arg(&so)
        .output()?;

    assert!(c.status.success());

    for path in dir.path().read_dir()? {
        println!("Entry: {}", path?.path().display())
    }

    let res = unsafe {
        let lib = Library::new(&so)?;
        let check_fn: Symbol<unsafe fn() -> c_int> = lib.get(b"test_sanity")?;

        check_fn()
    };

    assert_eq!(res, 99);
    Ok(())
}
