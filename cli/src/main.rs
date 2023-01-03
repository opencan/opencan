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
    Codegen(opencan_codegen::Args),
}

fn main() -> Result<()> {
    let args = PrimaryArgs::parse();

    // ugly ahh code
    match args.subcommand {
        Command::Compose(a) => opencan_compose::compose_entry(a).map(|_| ()),
        Command::Codegen(a) => Ok({
            let net = opencan_compose::compose_entry(opencan_compose::Args {
                in_file: a.in_file.clone(),
            })?;
            let out = opencan_codegen::Codegen::network_to_c(a, net)?;
            println!("{out}");
        }),
    }?;

    Ok(())
}
