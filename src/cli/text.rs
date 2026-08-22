use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::cli::{verify_file, verify_path};

#[derive(Debug, Subcommand)]
pub(crate) enum TextSubCommand {
    #[command(name = "sign", about = "Sign a message with a private/shared key")]
    Sign(TextSignOpts),

    #[command(name = "verify", about = "Verify a signed message")]
    Verify(TextVerifyOpts),

    #[command(name = "generate", about = "Generate a new key")]
    Generate(TextKeyGenerateOpts),

    #[command(name = "encrypt", about = "Encrypt a text")]
    Encrypt(TextEncryptOpts),

    #[command(name = "decrypt", about = "Decrypt a text")]
    Decrypt(TextDecryptOpts),
}

#[derive(Debug, Args)]
pub(crate) struct TextEncryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file )]
    pub key: String,
}

#[derive(Debug, Args)]
pub(crate) struct TextDecryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
}

#[derive(Debug, Args)]
pub(crate) struct TextSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file )]
    pub key: String,

    #[arg(long, default_value = "blake3")]
    pub format: TextSignFormat,
}

#[derive(Debug, Args)]
pub(crate) struct TextVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,

    #[arg(long)]
    pub sig: String,

    #[arg(long, default_value = "blake3")]
    pub format: TextSignFormat,
}

#[derive(Debug, Args)]
pub(crate) struct TextKeyGenerateOpts {
    #[arg(long, default_value = "blake3")]
    pub format: TextKeyFormat,

    #[arg(short, long, value_parser = verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TextKeyFormat {
    Blake3,
    Ed25519,
    Chacha20,
}

// fn parse_format(format: &str) -> Result<TextSignFormat, anyhow::Error> {
//     format.parse()
// }

// impl FromStr for TextSignFormat {
//     type Err = anyhow::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "blake3" => Ok(TextSignFormat::Blake3),
//             "ed25519" => Ok(TextSignFormat::Ed25519),
//             "chacha20" => Ok(TextSignFormat::Chacha20),
//             v => bail!("Unsupped type: {v}"),
//         }
//     }
// }

// impl From<TextSignFormat> for &'static str {
//     fn from(value: TextSignFormat) -> Self {
//         match value {
//             TextSignFormat::Blake3 => "blake3",
//             TextSignFormat::Ed25519 => "ed25519",
//             TextSignFormat::Chacha20 => "chacha20",
//         }
//     }
// }
