use anyhow::{Context, Result};
use clap::Parser;
use opencan_core::{CantoolsDecoder, TranslationLayer};

mod ymlfmt;
use ymlfmt::*;

mod translation;

#[derive(Parser)]
#[command(version)]
pub struct Args {
    in_file: String,
}

pub fn compose_entry(args: Args) -> Result<()> {
    let input = std::fs::read_to_string(&args.in_file).context("Failed to read input file")?;

    let de: YDesc = serde_yaml::from_str(&input).context(format!(
        "Failed to parse specification file `{}`",
        &args.in_file
    ))?;

    let net = match de.into_network() {
        Err(e) => {
            eprintln!("Failed to compose network.\n");
            eprintln!("What happened:");
            for (i, cause) in e.chain().enumerate() {
                eprintln!("`{} {}", "-".repeat(i), cause);
            }
            std::process::exit(-1);
        }

        Ok(net) => net,
    };

    println!("{}", serde_json::to_string_pretty(&net)?);
    println!("{}", CantoolsDecoder::dump_network(&net));
    Ok(())
}
