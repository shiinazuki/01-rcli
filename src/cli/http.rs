use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::cli::verify_path;

#[derive(Debug, Subcommand)]
pub(crate) enum HttpSubCommand {
    #[command(name = "serve", about = "Serve a directory over HTTP")]
    Serve(HttpServeOpts),

    #[command(name = "index", about = "Generate index.html for a directory tree")]
    Index(HttpIndexOpts),
}

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
