use std::{path::Path, process::Command};

use anyhow::{anyhow, Result};
use libloading::Library;
use tempfile::tempdir;

pub fn c_to_so(c_file: &Path) -> Result<Library> {
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

pub fn c_string_to_so(content: impl AsRef<[u8]>) -> Result<Library> {
    let temp_dir = tempdir()?;

    let dir = temp_dir.path();
    let c_file = dir.join("c_from_string.c");

    std::fs::write(&c_file, content)?;

    c_to_so(&c_file)
}
