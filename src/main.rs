// rcli csv -i input.csv -o output.json --header --d ','

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use rcli::{Opts, SubCommand, process_csv, process_genpass};

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
    }
    Ok(())
}
