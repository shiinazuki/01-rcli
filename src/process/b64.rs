use anyhow::Result;
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use tokio::io::AsyncReadExt;

use crate::{
    Base64Format::{self},
    InputReader, get_reader,
};

/// # Errors
pub async fn process_encode(input: &str, format: Base64Format) -> Result<String> {
    let mut reader = get_reader(input).await?;

    let mut buf = Vec::new();

    match &mut reader {
        InputReader::Stdin(stdin) => {
            stdin.read_to_end(&mut buf).await?;

            while let Some(&last) = buf.last() {
                if last == b'\n' || last == b'\r' {
                    buf.pop();
                } else {
                    break;
                }
            }
        }
        InputReader::File(file) => {
            file.read_to_end(&mut buf).await?;
        }
    }

    let encoded = match format {
        Base64Format::Standard => STANDARD.encode(buf),
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.encode(buf),
    };

    Ok(encoded)
}

/// # Errors
pub async fn process_decode(input: &str, format: Base64Format) -> Result<Vec<u8>> {
    let mut reader = get_reader(input).await?;

    let mut buf = Vec::new();
    match &mut reader {
        InputReader::Stdin(stdin) => stdin.read_to_end(&mut buf).await?,
        InputReader::File(file) => file.read_to_end(&mut buf).await?,
    };

    let input_str = String::from_utf8(buf)?;
    let clean_input = input_str.trim();

    let decoded = match format {
        Base64Format::Standard => STANDARD.decode(clean_input)?,
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.decode(clean_input)?,
    };

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_encode() {
        let input = "Cargo.toml";
        let format = Base64Format::Standard;
        assert!(process_encode(input, format).await.is_ok());
    }

    #[tokio::test]
    async fn test_process_decode() {
        let input = "fixtures/b64.txt";
        let format = Base64Format::Standard;
        assert!(process_decode(input, format).await.is_ok());
    }
}
