use clap::{Args, Subcommand, ValueEnum};

use crate::{CmdExecutor, cli::verify_file, process_decode, process_encode};

#[derive(Debug, Subcommand)]
pub(crate) enum Base64SubCommand {
    #[command(name = "encode", about = "Encode a string to base64")]
    Encode(Base64EncodeOpts),

    #[command(name = "decode", about = "Decode a  base64 to string")]
    Decode(Base64DecodeOpts),
}

impl_cmd_executor!(Base64SubCommand { Encode, Decode });

#[derive(Debug, Args)]
pub(crate) struct Base64EncodeOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(long, default_value = "standard")]
    pub format: Base64Format,
}

#[derive(Debug, Args)]
pub(crate) struct Base64DecodeOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(long, default_value = "standard")]
    pub format: Base64Format,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Base64Format {
    Standard,
    UrlSafe,
}

// fn parse_base64_format(format: &str) -> Result<Base64Format, anyhow::Error> {
//     format.parse()
// }

// impl From<Base64Format> for &'static str {
//     fn from(value: Base64Format) -> Self {
//         match value {
//             Base64Format::Standard => "standard",
//             Base64Format::UrlSafe => "urlsafe",
//         }
//     }
// }

// impl FromStr for Base64Format {
//     type Err = anyhow::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "standard" => Ok(Base64Format::Standard),
//             "urlsafe" => Ok(Base64Format::UrlSafe),
//             v => bail!("Unsupported format: {v}"),
//         }
//     }
// }

// 已使用宏优化
// impl CmdExecutor for Base64SubCommand {
//     async fn execute(self) -> anyhow::Result<()> {
//         match self {
//             Base64SubCommand::Encode(opts) => opts.execute().await?,
//             Base64SubCommand::Decode(opts) => opts.execute().await?,
//         }
//         Ok(())
//     }
// }

impl CmdExecutor for Base64DecodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decoded = process_decode(&self.input, self.format).await?;
        let decoded = String::from_utf8(decoded)?;
        println!("{decoded}");

        Ok(())
    }
}

impl CmdExecutor for Base64EncodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encoded = process_encode(&self.input, self.format).await?;
        println!("{encoded}");
        Ok(())
    }
}
