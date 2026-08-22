use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use tokio::fs;

use crate::{
    CmdExecutor,
    cli::{verify_file, verify_path},
    process_text_decrypt, process_text_encrypt, process_text_generate, process_text_sign,
    process_text_verify, write_secret,
};

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

impl_cmd_executor!(TextSubCommand {
    Sign,
    Verify,
    Generate,
    Encrypt,
    Decrypt
});

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

// 已使用宏优化
// impl CmdExecutor for TextSubCommand {
//     async fn execute(self) -> anyhow::Result<()> {
//         match self {
//             TextSubCommand::Sign(opts) => opts.execute().await?,
//             TextSubCommand::Verify(opts) => opts.execute().await?,
//             TextSubCommand::Generate(opts) => opts.execute().await?,
//             TextSubCommand::Encrypt(opts) => opts.execute().await?,
//             TextSubCommand::Decrypt(opts) => opts.execute().await?,
//         }
//         Ok(())
//     }
// }

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = process_text_sign(&self.input, &self.key, self.format).await?;
        println!("{signed}");
        Ok(())
    }
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verified = process_text_verify(&self.input, &self.key, self.format, &self.sig).await?;
        println!("{verified}");
        if !verified {
            std::process::exit(1);
        }
        Ok(())
    }
}

impl CmdExecutor for TextKeyGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let key = process_text_generate(self.format)?;
        match self.format {
            TextKeyFormat::Blake3 => {
                write_secret(self.output.join("blake3.txt"), &key[0]).await?;
            }
            TextKeyFormat::Ed25519 => {
                write_secret(self.output.join("ed25519.sk"), &key[0]).await?;
                fs::write(self.output.join("ed25519.pk"), &key[1]).await?;
            }
            TextKeyFormat::Chacha20 => {
                write_secret(self.output.join("chacha.txt"), &key[0]).await?;
            }
        }
        Ok(())
    }
}

impl CmdExecutor for TextEncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encrypt = process_text_encrypt(&self.input, &self.key).await?;
        println!("{encrypt}");
        Ok(())
    }
}

impl CmdExecutor for TextDecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decrypt = process_text_decrypt(&self.input, &self.key).await?;
        println!("{decrypt}");
        Ok(())
    }
}
