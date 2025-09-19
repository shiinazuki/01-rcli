use std::path::PathBuf;

use clap::Parser;

use crate::cli::verify_path;

#[derive(Debug, Parser)]
pub enum HttpSubCommand {
    #[command(about = "Serve a directory over HTTP")]
    Serve(HttpServeOpt),
}

#[derive(Debug, Parser)]
pub struct HttpServeOpt {
    #[arg(short, long, value_parser = verify_path, default_value = ".")]
    pub dir: PathBuf,

    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
}
