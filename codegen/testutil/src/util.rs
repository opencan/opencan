use std::{path::Path, process::Command};

use anyhow::{anyhow, Result};
use libloading::Library;
use tempfile::tempdir;

pub fn c_to_so(c_file: &[impl AsRef<Path>]) -> Result<Library> {
    let temp_dir = tempdir()?;

    let dir = temp_dir.path();
    let so = dir.join("lib.so");

    let mut gcc = Command::new("gcc");
    let c = gcc
        .arg("-Wall")
        .arg("-Wextra")
        // .arg("-Werror")
        .arg("-Wpedantic")
        .arg("-fPIC")
        .arg("-shared")
        .arg("-o")
        .arg(&so);

    for f in c_file {
        c.arg(f.as_ref());
    }

    let c = c.output()?;

    if !c.status.success() {
        return Err(anyhow!(
            "Failed compiling files:\n\n{}",
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

    c_to_so(&[&c_file])
}

pub fn c_strings_to_so<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<Library> {
    let temp_dir = tempdir()?;

    let dir = temp_dir.path();
    let mut sources = vec![];

    for (name, content) in files {
        let path = dir.join(name);
        std::fs::write(&dir.join(name), content)?;
        sources.push(path);
    }

    c_to_so(sources.as_slice())
}
