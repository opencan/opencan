use anyhow::Result;
use clap::Parser;
use opencan_compose::*;

fn main() -> Result<()> {
    let args = Args::parse();
    compose_entry(args)
}
