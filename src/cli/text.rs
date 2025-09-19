use anyhow::Result;
use std::{path::PathBuf, str::FromStr};

use clap::Parser;

use crate::cli::{verify_file, verify_path};

#[derive(Debug, Parser)]
pub enum TextSubCommand {
    #[command(about = "Sign a message with a private/shared key")]
    Sign(TextSignOpt),

    #[command(about = "Verify a signed message")]
    Verify(TextVerifyOpt),

    #[command(about = "Generate a new key")]
    Generate(TextKeyGenerateOpt),

    #[command(about = "encrypt txt")]
    Encrypt(EncryptOpt),

    #[command(about = "decrypt txt")]
    Decrypt(DecryptOpt),
}

#[derive(Debug, Parser)]
pub struct TextSignOpt {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(short, long, value_parser = verify_file)]
    pub key: PathBuf,

    #[arg(long, default_value = "blake3", value_parser = parse_text_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpt {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(short, long, value_parser = verify_file)]
    pub key: PathBuf,

    #[arg(short, long)]
    pub sign: String,

    #[arg(long, default_value = "blake3", value_parser = parse_text_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextKeyGenerateOpt {
    #[arg(short, long, default_value = "blake3", value_parser = parse_text_format)]
    pub format: TextSignFormat,

    #[arg(short, long, value_parser = verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct EncryptOpt {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(short, long , value_parser = verify_file)]
    pub key: PathBuf,

    #[arg(short, long, value_parser = verify_file)]
    pub nonce: PathBuf,
}

#[derive(Debug, Parser)]
pub struct DecryptOpt {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: PathBuf,

    #[arg(short, long , value_parser = verify_file)]
    pub key: PathBuf,

    #[arg(short, long, value_parser = verify_file)]
    pub nonce: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
    ChaCha20Poly1305,
}

fn parse_text_format(s: &str) -> Result<TextSignFormat, anyhow::Error> {
    s.parse()
}

impl std::fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            "chacha20poly1305" => Ok(TextSignFormat::ChaCha20Poly1305),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl From<TextSignFormat> for &'static str {
    fn from(value: TextSignFormat) -> Self {
        match value {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
            TextSignFormat::ChaCha20Poly1305 => "chacha20poly1305",
        }
    }
}
