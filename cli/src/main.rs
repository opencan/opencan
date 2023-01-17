use std::fs::{create_dir, write};
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Parser;
use opencan_codegen::{Codegen, CodegenOutput};

#[derive(clap::Parser)]
struct PrimaryArgs {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Compose a CAN network using a definitions file
    Compose(opencan_compose::Args),
    Codegen(opencan_codegen::Args),
}

fn main() -> Result<()> {
    let args = PrimaryArgs::parse();

    // ugly ahh code
    match args.subcommand {
        Command::Compose(a) => opencan_compose::compose(a).map(|_| ()),
        Command::Codegen(a) => {
            let net = opencan_compose::compose(opencan_compose::Args {
                in_file: a.in_file.clone(),
            })?;
            let gen = Codegen::new(a, &net);
            let out = gen.network_to_c()?;
            save_codegen_files(&out, "./codegenoutput")?;
            Ok(())
        }
    }
}

/// Save output files from codegen to given path.
fn save_codegen_files(cg: &CodegenOutput, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        create_dir(path)?;
    } else if !path.is_dir() {
        return Err(anyhow!(
            "Can't save codegen output into `{}` - is not a directory",
            path.display()
        ));
    }

    for (name, content) in cg.as_list() {
        write(path.join(name), content)?;
    }

    Ok(())
}
