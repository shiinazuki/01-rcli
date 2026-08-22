use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::{CmdExecutor, cli::verify_path, process_http_index, process_http_serve};

#[derive(Debug, Subcommand)]
pub(crate) enum HttpSubCommand {
    #[command(name = "serve", about = "Serve a directory over HTTP")]
    Serve(HttpServeOpts),

    #[command(name = "index", about = "Generate index.html for a directory tree")]
    Index(HttpIndexOpts),
}

impl_cmd_executor!(HttpSubCommand { Serve, Index });

#[derive(Debug, Args)]
pub(crate) struct HttpServeOpts {
    #[arg(long, value_parser = verify_path, default_value = ".")]
    pub dir: PathBuf,

    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Debug, Args)]
pub(crate) struct HttpIndexOpts {
    #[arg(long, value_parser = verify_path, default_value = ".")]
    pub dir: PathBuf,

    #[arg(long, default_value_t = false, help = "Overwrite existing index.html")]
    pub force: bool,
}

// 已使用宏优化
// impl CmdExecutor for HttpSubCommand {
//     async fn execute(self) -> anyhow::Result<()> {
//         match self {
//             HttpSubCommand::Serve(opts) => opts.execute().await?,
//             HttpSubCommand::Index(opts) => opts.execute().await?,
//         }
//         Ok(())
//     }
// }

impl CmdExecutor for HttpServeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        process_http_serve(self.dir, self.port).await
    }
}

impl CmdExecutor for HttpIndexOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let n = process_http_index(self.dir, self.force).await?;
        println!("Generated {n} index.html file(s)");
        Ok(())
    }
}
