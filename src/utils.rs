use std::{
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncRead, AsyncWriteExt, ReadBuf},
};

#[derive(Debug)]
pub enum InputReader {
    Stdin(io::Stdin),
    File(File),
}

/// # Errors
pub async fn write_secret(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    opts.mode(0o600);

    let mut file = opts.open(path).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .await?;
    }

    file.write_all(contents).await?;
    file.flush().await?;
    Ok(())
}

impl AsyncRead for InputReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            InputReader::Stdin(stdin) => Pin::new(stdin).poll_read(cx, buf),
            InputReader::File(file) => Pin::new(file).poll_read(cx, buf),
        }
    }
}

/// # Errors
pub async fn get_reader(input: &str) -> Result<InputReader> {
    let reader = if input == "-" {
        InputReader::Stdin(io::stdin())
    } else {
        InputReader::File(File::open(input).await?)
    };
    Ok(reader)
}
