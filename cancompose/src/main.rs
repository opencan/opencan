use anyhow::{Context, Result};
use clap::Parser;

mod ymlfmt;
use ymlfmt::*;

mod translation;

#[derive(Parser)]
#[command(version)]
struct Args {
    in_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = std::fs::read_to_string(&args.in_file).context("Failed to read input file")?;

    let de: YDesc = serde_yaml::from_str(&input).context(format!(
        "Failed to parse specification file `{}`",
        &args.in_file
    ))?;

    let try_net = de.into_network();

    if let Err(err) = try_net {
        eprintln!("Failed to compose network.\n");
        eprintln!("What happened:");
        for (i, cause) in err.chain().enumerate() {
            eprintln!("`{} {}", "-".repeat(i), cause);
        }
        std::process::exit(-1);
    }

    let net = try_net.unwrap();

    println!("{}", serde_json::to_string_pretty(&net)?);
    Ok(())
}
