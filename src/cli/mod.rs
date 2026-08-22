mod base64;
mod csv;
mod genpass;
mod http;
mod text;

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

pub use self::{
    base64::Base64Format,
    csv::OutputFormat,
    text::{TextKeyFormat, TextSignFormat},
};
pub(crate) use self::{base64::Base64SubCommand, http::HttpSubCommand, text::TextSubCommand};
use crate::cli::{csv::CsvOpts, genpass::GenPassOpts};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub(crate) cmd: SubCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SubCommand {
    #[command(name = "csv", about = "Show CSV, or convert CSV to other format")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64SubCommand),

    #[command(subcommand)]
    Text(TextSubCommand),

    #[command(subcommand)]
    Http(HttpSubCommand),
}

impl_cmd_executor!(SubCommand {
    Csv,
    GenPass,
    Base64,
    Text,
    Http
});

// 已使用宏优化
// impl CmdExecutor for SubCommand {
//     async fn execute(self) -> Result<()> {
//         match self {
//             SubCommand::Csv(opts) => opts.execute().await?,
//             SubCommand::GenPass(opts) => opts.execute().await?,
//             SubCommand::Base64(subcmd) => subcmd.execute().await?,
//             SubCommand::Text(subcmd) => subcmd.execute().await?,
//             SubCommand::Http(subcmd) => subcmd.execute().await?,
//         }
//         Ok(())
//     }
// }

fn verify_file(filename: &str) -> Result<String, &'static str> {
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exist")
    }
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("Path does not exist or is not a directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("*"), Err("File does not exist"));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_file("not-exist"), Err("File does not exist"));
    }
}
