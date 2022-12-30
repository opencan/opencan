use anyhow::Result;
use cancompose::*;
use clap::Parser;

fn main() -> Result<()> {
    let args = Args::parse();
    compose_entry(args)
}
