// rcli csv -i input.csv -o output.json --header --d ','

use anyhow::Result;
use clap::Parser;
use rcli::{Opts, SubCommand, process_csv};

fn main() -> Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opt) => process_csv(opt.input, opt.output)?,
    }
    Ok(())
}
