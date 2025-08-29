use std::{path::PathBuf, str::FromStr};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV or convert csv to other format")]
    Csv(CsvOpt),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpt),
}

#[derive(Debug, Parser)]
pub struct CsvOpt {
    #[arg(short, long, value_parser = verify_input_file)]
    pub input: PathBuf,

    #[arg(short, long,  value_parser = verify_output_file)]
    pub output: Option<PathBuf>,

    #[arg(long, default_value = "json", value_parser = parse_format)]
    pub format: OutputFormat,

    #[arg(short, long, default_value_t = ',')]
    pub deliment: char,

    #[arg(long, default_value_t = true)]
    pub header: bool,
}

#[derive(Debug, Parser)]
pub struct GenPassOpt {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(long)]
    pub no_uppercase: bool,

    #[arg(long)]
    pub no_lowercase: bool,

    #[arg(long)]
    pub no_number: bool,

    #[arg(long)]
    pub no_symbol: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
}

impl From<OutputFormat> for &'static str {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Json => "json",
            OutputFormat::Yaml => "yaml",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

fn parse_format(format: &str) -> Result<OutputFormat, anyhow::Error> {
    format.parse()
}

fn verify_input_file(filename: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(filename);
    if !path.exists() {
        return Err(format!("file is not exists: {}", filename));
    }
    if !path.is_file() {
        return Err(format!("path is not a file: {}", filename));
    }
    Ok(path)
}

fn verify_output_file(filename: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(filename);
    if path.is_dir() {
        return Err(format!("path {} is a directory", path.display()));
    }
    let parent = path.parent();

    let needs_parent_validation = parent.is_some() && !parent.unwrap().as_os_str().is_empty();

    if needs_parent_validation {
        let parent_path = parent.unwrap();

        if !parent_path.exists() {
            return Err(format!(
                "output drectory {} is not exists",
                parent_path.display()
            ));
        }

        if !parent_path.is_dir() {
            return Err(format!("{} the parent is not drectory", path.display()));
        }
    }

    Ok(path)
}
