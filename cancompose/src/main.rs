use std::io::{self, BufRead};

use anyhow::{Context, Result};
use can::{CantoolsDecoder, TranslationLayer};
use clap::Parser;

mod ymlfmt;
use ymlfmt::*;

mod translation;

#[derive(Parser)]
#[command(version)]
struct Args {
    in_file: String,
    // msg_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

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
    // println!("{}", CantoolsDecoder::dump_network(&net));

    println!("Ok, decoding...");
    // println!(
    //     "{}",
    //     net.message_by_name(&args.msg_name)
    //         .unwrap()
    //         .decode_string(0xFFFFFFFF)
    // );

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(line) = lines.next() {
        let line = line.unwrap();
        let cols: Vec<&str> = line.split_whitespace().collect();

        // cols[0] is the can iface
        // cols[1] is the message id
        let message_id: u32 = cols[1].parse().unwrap();

        // cols[2] is the [dlc]
        let mut i = cols[2].chars().into_iter();
        i.next();
        i.next_back();

        let dlc: u8 = i.collect::<String>().parse().unwrap();

        let data: String = cols.into_iter().skip(3).collect();
        let data_padded = format!("{:0>16}", data);
        println!("{message_id}: [{dlc}] {data_padded}");

        let data_raw = u64::from_str_radix(&data_padded, 16).unwrap();

        println!(
            "{}",
            net.message_by_id(&message_id)
                .unwrap()
                .decode_string(data_raw)
        );
    }
    Ok(())
}
