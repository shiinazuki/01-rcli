mod base64;
mod csv;
mod genpass;

use std::path::PathBuf;

pub use self::{
    base64::{Base64Format, Base64SubCommand, InputReader},
    csv::OutputFormat,
};

use self::{csv::CsvOpt, genpass::GenPassOpt};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV or convert csv to other format")]
    Csv(CsvOpt),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpt),

    #[command(subcommand)]
    Base64(Base64SubCommand),
}

fn verify_input_file(filename: &str) -> Result<PathBuf, String> {
    if filename == "-" {
        return Ok(PathBuf::from("-"));
    }
    let path = PathBuf::from(filename);
    if !path.exists() {
        return Err(format!("file is not exists: {}", filename));
    }
    if !path.is_file() {
        return Err(format!("path is not a file: {}", filename));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_input_file("-"), Ok(PathBuf::from("-")));
        assert_eq!(
            verify_input_file("Cargo.toml"),
            Ok(PathBuf::from("Cargo.toml"))
        );
        assert_eq!(
            verify_input_file("not_exist"),
            Err("file is not exists: not_exist".to_string())
        );
    }
}
