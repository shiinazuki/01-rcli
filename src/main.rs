// rcli csv -i input.csv -o output.json --header --d ','

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use rcli::{
    Base64SubCommand, Opts, SubCommand, process_csv, process_decode, process_encode,
    process_genpass,
};

fn main() -> Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opt) => {
            let output = if let Some(output) = opt.output {
                output.clone()
            } else {
                PathBuf::from(format!("output.{}", opt.format))
            };
            process_csv(opt.input, output, opt.format)?;
        }

        SubCommand::GenPass(opt) => {
            process_genpass(
                opt.length,
                opt.no_uppercase,
                opt.no_lowercase,
                opt.no_number,
                opt.no_symbol,
            )?;
        }

        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opt) => {
                process_encode(opt.input, opt.format)?;
            }
            Base64SubCommand::Decode(opt) => {
                process_decode(opt.input, opt.format)?;
            }
        },
    }
    Ok(())
}
