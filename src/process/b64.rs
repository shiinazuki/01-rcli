use std::{io::Read, path::PathBuf};

use anyhow::Result;
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};

use crate::{Base64Format, InputReader};

pub fn process_encode(input: PathBuf, format: Base64Format) -> Result<()> {
    let mut reader = InputReader::from_path(input)?;

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let engine = match format {
        Base64Format::Standard => &STANDARD,
        Base64Format::UrlSafe => &URL_SAFE_NO_PAD,
    };

    let encoded = engine.encode(&buf);

    println!("{}", encoded);
    Ok(())
}

pub fn process_decode(input: PathBuf, format: Base64Format) -> Result<()> {
    let mut reader = InputReader::from_path(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let engine = match format {
        Base64Format::Standard => &STANDARD,
        Base64Format::UrlSafe => &URL_SAFE_NO_PAD,
    };

    let decoded = engine.decode(buf.trim())?;
    let decoded = String::from_utf8(decoded)?;
    println!("{}", decoded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let input = PathBuf::from("Cargo.toml");
        let format = Base64Format::UrlSafe;
        assert!(process_encode(input, format).is_ok());
    }

    #[test]
    fn test_decode() {
        let input = PathBuf::from("fixtures/b64.txt");
        let format = Base64Format::UrlSafe;
        assert!(process_decode(input, format).is_ok());
    }
}
