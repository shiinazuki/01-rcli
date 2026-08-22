use clap::{Args, value_parser};
use zxcvbn::zxcvbn;

use crate::{CmdExecutor, process_genpass};

#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI 开关天然就是一组彼此独立的 bool 标志，拆成枚举反而更难用"
)]
#[derive(Debug, Args)]
pub(crate) struct GenPassOpts {
    #[arg(short, long, default_value_t = 16, value_parser = value_parser!(u8).range(8..))]
    pub length: u8,

    #[arg(long, default_value_t = false)]
    pub no_uppercase: bool,

    #[arg(long, default_value_t = false)]
    pub no_lowercase: bool,

    #[arg(long, default_value_t = false)]
    pub no_number: bool,

    #[arg(long, default_value_t = false)]
    pub no_symbol: bool,
}

#[expect(clippy::unused_async_trait_impl)]
impl CmdExecutor for GenPassOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let password = process_genpass(
            self.length,
            self.no_uppercase,
            self.no_lowercase,
            self.no_number,
            self.no_symbol,
        )?;
        println!("{password}");
        let result = zxcvbn(&password, &[]);
        eprintln!("Password strength: {}", result.score());
        Ok(())
    }
}
