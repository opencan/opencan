//! Composition of [`CANNetwork`]s from YAML definition files.

use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use opencan_core::{translation::CantoolsTranslator, CANNetwork, TranslationFromOpencan};

mod ymlfmt;
use ymlfmt::*;

mod translation;

#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Input .yml file
    pub in_file: String,

    /// Dump composed network as JSON to stdout
    #[clap(long, short, action)]
    pub dump_json: bool,

    /// Dump composed network as Python to stdout
    #[clap(long, action)]
    pub dump_python: bool,
}

/// Compose YAML definitions into a `CANNetwork` given opencan_compose::Args.
pub fn compose(args: Args) -> Result<CANNetwork> {
    let input = fs::read_to_string(&args.in_file).context("Failed to read input file")?;

    let net = compose_str(&input).context(format!(
        "Failed to ingest specifications file {}",
        args.in_file
    ))?;

    if args.dump_json {
        println!("{}", serde_json::to_string_pretty(&net).unwrap());
    }

    if args.dump_python {
        println!("{}", CantoolsTranslator::translate(&net));
    }

    Ok(net)
}

/// Compose YAML definitions from a `&str` directly.
pub fn compose_str(input: &str) -> Result<CANNetwork> {
    let de: YDesc =
        serde_yaml::from_str(input).context("Failed to parse specifications.".to_string())?;

    // We manually print out the error causes so we can use our own formatting rather than anyhow's.
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

    Ok(net)
}
