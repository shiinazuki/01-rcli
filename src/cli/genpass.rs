use clap::Parser;

#[derive(Debug, Parser)]
pub struct GenPassOpt {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(long)]
    pub no_uppercase: bool,

    #[arg(long)]
    pub no_lowercase: bool,

    #[arg(long)]
    pub no_number: bool,

    #[arg(long)]
    pub no_symbol: bool,
}
