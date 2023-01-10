use anyhow::Result;
use clap::Parser;
use opencan_codegen::Codegen;

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
        Command::Codegen(a) => {
            let net = opencan_compose::compose_entry(opencan_compose::Args {
                in_file: a.in_file.clone(),
            })?;
            let gen = Codegen::new(a, net);
            let out = gen.network_to_c()?;
            println!("{out}");
            Ok(())
        }
    }
}
