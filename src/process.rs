use std::{fs, path::PathBuf};

use anyhow::Result;
use csv::Reader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

pub fn process_csv(input: PathBuf, output: PathBuf) -> Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    for result in reader.deserialize() {
        let record: Player = result?;
        ret.push(record);
    }
    let record_json = serde_json::to_string_pretty(&ret)?;
    fs::write(output, record_json)?;
    Ok(())
}
