use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Stdin},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

use crate::cli::verify_input_file;

#[derive(Debug, Parser)]
pub enum Base64SubCommand {
    #[command(name = "encode", about = "Encode base64")]
    Encode(Base64EncodeOpt),

    #[command(name = "decode", about = "Decode base64")]
    Decode(Base64DecodeOpt),
}

#[derive(Debug, Parser)]
pub struct Base64EncodeOpt {
    #[arg(short, long, value_parser = verify_input_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(long, value_parser = parse_base64_format,  default_value = "standard")]
    pub format: Base64Format,
}

#[derive(Debug, Parser)]
pub struct Base64DecodeOpt {
    #[arg(short, long, value_parser = verify_input_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(long, value_parser = parse_base64_format, default_value = "standard")]
    pub format: Base64Format,
}

#[derive(Debug, Clone, Copy)]
pub enum Base64Format {
    Standard,
    UrlSafe,
}

pub enum InputReader {
    Stdin(Stdin),
    File(File),
}

impl InputReader {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if path.as_os_str() == "-" {
            // Box::new(std::io::stdin())
            Ok(InputReader::Stdin(std::io::stdin()))
        } else {
            // Box::new(File::open(input)?)
            Ok(InputReader::File(File::open(path)?))
        }
    }
}

impl Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputReader::Stdin(stdin) => stdin.read(buf),
            InputReader::File(file) => file.read(buf),
        }
    }
}

fn parse_base64_format(format: &str) -> Result<Base64Format, anyhow::Error> {
    format.parse()
}

impl FromStr for Base64Format {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Base64Format::Standard),
            "urlsafe" => Ok(Base64Format::UrlSafe),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl std::fmt::Display for Base64Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl From<Base64Format> for &'static str {
    fn from(value: Base64Format) -> Self {
        match value {
            Base64Format::Standard => "standard",
            Base64Format::UrlSafe => "urlsafe",
        }
    }
}
