//! Composition of [`CANNetwork`]s from YAML definition files.

use anyhow::{Context, Result};
use clap::Parser;
use opencan_core::CANNetwork;

mod ymlfmt;
use ymlfmt::*;

mod translation;

#[derive(Parser)]
#[command(version)]
pub struct Args {
    pub in_file: String,
}

/// Compose YAML definitions into a `CANNetwork` given opencan_compose::Args.
pub fn compose(args: Args) -> Result<CANNetwork> {
    let input = std::fs::read_to_string(&args.in_file).context("Failed to read input file")?;

    compose_str(&input).context(format!(
        "Failed to ingest specifications file {}",
        args.in_file
    ))
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

    eprintln!("{}", serde_json::to_string_pretty(&net).unwrap());

    Ok(net)
}
