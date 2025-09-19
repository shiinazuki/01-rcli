use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Stdin},
    path::PathBuf,
};

pub enum InputReader {
    Stdin(Stdin),
    File(File),
}

impl InputReader {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if path.as_os_str() == "-" {
            // Box::new(std::io::stdin())
            Ok(InputReader::Stdin(std::io::stdin()))
        } else {
            // Box::new(File::open(input)?)
            Ok(InputReader::File(File::open(path)?))
        }
    }
}

impl Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputReader::Stdin(stdin) => stdin.read(buf),
            InputReader::File(file) => file.read(buf),
        }
    }
}
