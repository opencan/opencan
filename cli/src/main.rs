use anyhow::Result;
use clap::Parser;

#[derive(clap::Parser)]
struct PrimaryArgs {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Compose a CAN network using a definitions file
    Compose(opencan_compose::Args),
}

fn main() -> Result<()> {
    let args = PrimaryArgs::parse();

    match args.subcommand {
        Command::Compose(a) => opencan_compose::compose_entry(a)?,
    }

    Ok(())
}
