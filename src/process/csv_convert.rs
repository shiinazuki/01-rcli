use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;

use crate::cli::OutputFormat;

#[expect(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

/// # Errors
pub async fn process_csv(
    input: &str,
    output: String,
    format: OutputFormat,
    delimiter: char,
    has_header: bool,
) -> Result<()> {
    let delimiter = u8::try_from(delimiter)
        .ok()
        .filter(u8::is_ascii)
        .with_context(|| {
            format!("delimiter must be a single ASCII character, got {delimiter:?}")
        })?;

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(has_header)
        .from_path(input)?;

    let headers: Vec<String> = if has_header {
        reader.headers()?.iter().map(str::to_owned).collect()
    } else {
        (1..=reader.headers()?.len())
            .map(|i| format!("col{i}"))
            .collect()
    };

    let records = reader
        .records()
        .map(|record| {
            let record = record?;
            let json_value = headers
                .iter()
                .map(String::as_str)
                .zip(record.iter())
                .collect();
            Ok::<Value, csv::Error>(json_value)
        })
        .collect::<Result<Vec<Value>, _>>()
        .context("failed to deserialize csv")?;

    let json = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&records)?,
        OutputFormat::Yaml => serde_saphyr::to_string(&records)?,
        OutputFormat::Toml => {
            let mut root = serde_json::Map::new();
            root.insert("players".into(), Value::Array(records));
            let value = Value::Object(root);
            toml::to_string_pretty(&value)?
        }
    };

    fs::write(output, json).await?;
    Ok(())
}
