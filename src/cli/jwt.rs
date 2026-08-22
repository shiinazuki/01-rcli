use std::time::Duration;

use anyhow::{Result, bail};
use clap::{Args, Subcommand};

use crate::{
    CmdExecutor, cli::verify_file, process_jwt_pubkey, process_jwt_sign, process_jwt_verify,
};

#[derive(Debug, Subcommand)]
pub(crate) enum JwtSubCommand {
    #[command(name = "sign", about = "Sign a JWT with an Ed25519 private key")]
    Sign(JwtSignOpts),

    #[command(name = "verify", about = "Verify a JWT with an Ed25519 public key")]
    Verify(JwtVerifyOpts),

    #[command(
        name = "pubkey",
        about = "Print the public key as SPKI PEM (for jwt.io)"
    )]
    Pubkey(JwtPubkeyOpts),
}

impl_cmd_executor!(JwtSubCommand {
    Sign,
    Verify,
    Pubkey
});

#[derive(Debug, Args)]
pub(crate) struct JwtSignOpts {
    #[arg(long, value_parser = verify_file)]
    pub key: String,

    #[arg(long)]
    pub sub: String,

    #[arg(long)]
    pub aud: String,

    #[arg(long, value_parser = parse_exp, default_value = "14d")]
    pub exp: Duration,
}

#[derive(Debug, Args)]
pub(crate) struct JwtVerifyOpts {
    #[arg(long, value_parser = verify_file)]
    pub key: String,

    #[arg(short, long)]
    pub token: String,

    #[arg(long)]
    pub aud: Option<String>,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> Result<()> {
        let token = process_jwt_sign(&self.key, &self.sub, &self.aud, self.exp).await?;
        println!("{token}");
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> Result<()> {
        let claims = process_jwt_verify(&self.key, &self.token, self.aud.as_deref()).await?;
        println!("{}", serde_json::to_string_pretty(&claims)?);
        Ok(())
    }
}

fn parse_exp(exp: &str) -> Result<Duration> {
    let duration = humantime::parse_duration(exp)?;
    if duration.is_zero() {
        bail!("expiration must be greater than zero");
    }
    Ok(duration)
}

#[derive(Debug, Args)]
pub(crate) struct JwtPubkeyOpts {
    #[arg(long, value_parser = verify_file)]
    pub key: String,
}

impl CmdExecutor for JwtPubkeyOpts {
    async fn execute(self) -> Result<()> {
        print!("{}", process_jwt_pubkey(&self.key).await?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exp_accepts_common_forms() {
        assert_eq!(
            parse_exp("14d").unwrap(),
            Duration::from_secs(14 * 24 * 3600)
        );
        assert_eq!(parse_exp("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_exp("30m").unwrap(), Duration::from_secs(1800));
    }

    #[test]
    fn parse_exp_rejects_zero_and_garbage() {
        assert!(parse_exp("0s").is_err());
        assert!(parse_exp("banana").is_err());
        assert!(parse_exp("").is_err());
    }
}
