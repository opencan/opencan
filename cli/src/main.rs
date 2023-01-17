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
    Codegen {
        /// Input .yml file
        in_file: String,
        /// Output directory (created if it doesn't exist yet)
        output_path: String,
        /// Codegen arguments
        #[clap(flatten)]
        cg_args: opencan_codegen::Args,
    },
}

fn main() -> Result<()> {
    let args = PrimaryArgs::parse();

    // for now, we call compose every time we call codegen because we don't
    // have reliable serialization/deserialization of the network from core.
    //
    // now:
    // 1. codegen <- compose <- yml
    //
    // later:
    // 1. (compose <- yml) -> network.json
    // 2. codegen <- network.json
    match args.subcommand {
        Command::Compose(a) => opencan_compose::compose(a).map(|_| ()),
        Command::Codegen {
            cg_args,
            in_file,
            output_path,
        } => {
            let net = opencan_compose::compose(opencan_compose::Args {
                in_file,
                dump_json: false,
            })?;
            let gen = Codegen::new(cg_args, &net);
            let out = gen.network_to_c()?;
            save_codegen_files(&out, output_path)?;
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
