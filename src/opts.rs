use std::path::PathBuf;

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
}

#[derive(Debug, Parser)]
pub struct CsvOpt {
    #[arg(short, long, value_parser = verify_input_file)]
    pub input: PathBuf,

    #[arg(short, long, default_value = "output.json", value_parser = verify_output_file)]
    pub output: PathBuf,

    #[arg(short, long, default_value_t = ',')]
    pub deliment: char,

    #[arg(long, default_value_t = true)]
    pub header: bool,
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
