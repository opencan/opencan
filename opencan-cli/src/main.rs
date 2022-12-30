use anyhow::Result;
use clap::Parser;

#[derive(clap::Parser)]
struct PrimaryArgs {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Compose(cancompose::Args),
}

fn main() -> Result<()> {
    let args = PrimaryArgs::parse();

    match args.subcommand {
        Command::Compose(a) => cancompose::compose_entry(a)?,
    }

    Ok(())
}
