use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rng;

use std::{fs, io::Read, path::PathBuf};

use anyhow::Result;

use crate::{InputReader, TextSignFormat, process_genpass};

pub trait TextSign {
    fn sign<R: Read>(&self, reader: &mut R) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify<R: Read>(&self, reader: R, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    fn load(path: PathBuf) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub trait TextEncrypt {
    fn encrypt<R: Read>(&self, reader: R) -> Result<Vec<u8>>;
}

pub trait TextDecrypt {
    fn decrypt<R: Read>(&self, reader: R) -> Result<Vec<u8>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = key.try_into().unwrap();
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl TextSign for Blake3 {
    fn sign<R: Read>(&self, reader: &mut R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify<R: Read>(&self, mut reader: R, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: PathBuf) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, false, false, false, false)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

pub struct Ed25519Signer {
    key: SigningKey,
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let signer = Ed25519Signer::new(key);
        Ok(signer)
    }
}

impl TextSign for Ed25519Signer {
    fn sign<R: Read>(&self, reader: &mut R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: PathBuf) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut rng = rng();
        let sk = SigningKey::generate(&mut rng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();

        Ok(vec![sk, pk])
    }
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        let signer = Ed25519Verifier::new(key);
        Ok(signer)
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify<R: Read>(&self, mut reader: R, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();

        Ok(ret)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: PathBuf) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

pub struct Chacha20Poly1305Text {
    cipher: ChaCha20Poly1305,
    nonce: Nonce,
}

impl Chacha20Poly1305Text {
    pub fn new(cipher: ChaCha20Poly1305, nonce: Nonce) -> Self {
        Self { cipher, nonce }
    }

    pub fn try_new(key: &[u8], nonce: &[u8]) -> Result<Self> {
        let cipher = ChaCha20Poly1305::new(key.try_into()?);
        let nonce = nonce.try_into()?;
        Ok(Self::new(cipher, nonce))
    }
}

impl TextEncrypt for Chacha20Poly1305Text {
    fn encrypt<R: Read>(&self, mut reader: R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let encrypt = self.cipher.encrypt(&self.nonce, buf.as_ref())?;
        Ok(encrypt)
    }
}

impl TextDecrypt for Chacha20Poly1305Text {
    fn decrypt<R: Read>(&self, mut reader: R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let decrypt = self.cipher.decrypt(&self.nonce, buf.as_ref())?;
        Ok(decrypt)
    }
}

impl KeyGenerator for Chacha20Poly1305Text {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut rng = rand::rng();
        let key = ChaCha20Poly1305::generate_key_with_rng(&mut rng);
        let nonce = ChaCha20Poly1305::generate_nonce_with_rng(&mut rng);
        Ok(vec![key.to_vec(), nonce.to_vec()])
    }
}

pub fn process_text_sign(input: PathBuf, key: PathBuf, format: TextSignFormat) -> Result<String> {
    let mut reader = InputReader::from_path(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let reader = buf.trim();

    let signed = match format {
        TextSignFormat::Blake3 => {
            let blake3 = Blake3::load(key)?;
            blake3.sign(&mut reader.as_bytes())?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader.as_bytes())?
        }
        _ => todo!(),
    };

    let signed = URL_SAFE_NO_PAD.encode(&signed);

    Ok(signed)
}

pub fn process_text_verify(
    input: PathBuf,
    key: PathBuf,
    format: TextSignFormat,
    sig: String,
) -> Result<bool> {
    let mut reader = InputReader::from_path(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig.trim())?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let reader = buf.trim();

    let ret = match format {
        TextSignFormat::Blake3 => {
            let blake3 = Blake3::load(key)?;
            blake3.verify(reader.as_bytes(), &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verify = Ed25519Verifier::load(key)?;
            verify.verify(reader.as_bytes(), &sig)?
        }
        _ => todo!(),
    };
    Ok(ret)
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
        TextSignFormat::ChaCha20Poly1305 => Chacha20Poly1305Text::generate(),
    }
}

pub fn process_text_encrypt(input: PathBuf, key: PathBuf, nonce: PathBuf) -> Result<String> {
    let mut reader = InputReader::from_path(input)?;
    let key = fs::read(key)?;
    let nonce = fs::read(nonce)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let plaintext = buf.trim();
    let clipher = Chacha20Poly1305Text::try_new(&key, &nonce)?;
    let cliphertext = URL_SAFE_NO_PAD.encode(clipher.encrypt(&mut plaintext.as_bytes())?);
    Ok(cliphertext)
}

pub fn process_text_decrypt(input: PathBuf, key: PathBuf, nonce: PathBuf) -> Result<String> {
    let mut reader = InputReader::from_path(input)?;

    let key = fs::read(key)?;
    let nonce = fs::read(nonce)?;
    let clipher = Chacha20Poly1305Text::try_new(&key, &nonce)?;

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let ciphertext = URL_SAFE_NO_PAD.decode(buf.trim())?;

    let cliphertext = String::from_utf8(clipher.decrypt(&mut ciphertext.as_slice())?)?;
    Ok(cliphertext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let blake3 = Blake3::load(PathBuf::from("fixtures/blake3.txt"))?;

        let data = b"hello world";
        let sig = blake3.sign(&mut &data[..]).unwrap();
        assert!(blake3.verify(&data[..], &sig).unwrap());

        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let sk = Ed25519Signer::load(PathBuf::from("fixtures/ed25519.sk"))?;
        let pk = Ed25519Verifier::load(PathBuf::from("fixtures/ed25519.pk"))?;

        let data = b"hello world";
        let sig = sk.sign(&mut &data[..]).unwrap();
        assert!(pk.verify(&data[..], &sig)?);

        Ok(())
    }
}
