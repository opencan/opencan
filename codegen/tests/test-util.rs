use std::ffi::c_int;

use anyhow::Result;
use libloading::Symbol;
use pyo3::prelude::*;
use testutil::util::*;

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
