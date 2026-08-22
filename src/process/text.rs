use std::path::Path;

use anyhow::{Ok, Result, anyhow};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit, Nonce,
    aead::{Aead, Generate, Key},
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rand_core::UnwrapErr, rngs::SysRng};
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
};

use crate::{
    cli::{TextKeyFormat, TextSignFormat},
    get_reader,
    process::gen_pass,
};

const NONCE_LEN: usize = 12;

trait TextSign {
    async fn sign(&self, reader: impl AsyncRead) -> Result<Vec<u8>>;
}

trait TextVerify {
    async fn verify(&self, reader: impl AsyncRead, sig: &[u8]) -> Result<bool>;
}

trait KeyLoader {
    async fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

trait TextEncrypt {
    async fn encrypt(&self, reader: impl AsyncRead) -> Result<Vec<u8>>;
}

trait TextDecrypt {
    fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>>;
}

struct Chacha20 {
    key: [u8; 32],
}

struct Blake3 {
    key: [u8; 32],
}

struct Ed25519Signer {
    key: SigningKey,
}

struct Ed25519Verifier {
    key: VerifyingKey,
}

/// # Errors
pub async fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let reader = get_reader(input).await?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key).await?;
            signer.sign(reader).await?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key).await?;
            signer.sign(reader).await?
        }
    };

    let signed = URL_SAFE_NO_PAD.encode(&signed);

    Ok(signed)
}

/// # Errors
pub async fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> Result<bool> {
    let mut reader = get_reader(input).await?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key).await?;
            verifier.verify(&mut reader, &sig).await?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key).await?;
            verifier.verify(&mut reader, &sig).await?
        }
    };

    Ok(verified)
}

/// # Errors
pub fn process_text_generate(format: TextKeyFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextKeyFormat::Blake3 => Blake3::generate(),
        TextKeyFormat::Ed25519 => Ed25519Signer::generate(),
        TextKeyFormat::Chacha20 => Chacha20::generate(),
    }
}

/// # Errors
pub async fn process_text_encrypt(input: &str, key: &str) -> Result<String> {
    let reader = get_reader(input).await?;
    let cipher = Chacha20::load(key).await?;
    let encrypted = cipher.encrypt(reader).await?;
    Ok(URL_SAFE_NO_PAD.encode(&encrypted))
}

/// # Errors
pub async fn process_text_decrypt(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input).await?;
    let mut b64 = Vec::new();
    reader.read_to_end(&mut b64).await?;
    let payload = URL_SAFE_NO_PAD.decode(b64.trim_ascii())?;

    let cipher = Chacha20::load(key).await?;
    let plaintext = cipher.decrypt(&payload)?;
    Ok(String::from_utf8(plaintext)?)
}

impl Chacha20 {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = key.try_into().map_err(|_| anyhow!("长度{}", key.len()))?;

        let signer = Chacha20::new(key);
        Ok(signer)
    }
}

impl TextEncrypt for Chacha20 {
    async fn encrypt(&self, reader: impl AsyncRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        tokio::pin!(reader);
        reader.read_to_end(&mut buf).await?;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| anyhow!("chacha20poly1305 密钥长度必需是 32 字节"))?;

        let nonce = Nonce::generate();

        let ciphertext = cipher
            .encrypt(&nonce, buf.as_ref())
            .map_err(|e| anyhow!("加密失败: {e}"))?;

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }
}

impl TextDecrypt for Chacha20 {
    fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < NONCE_LEN {
            return Err(anyhow!(
                "密文太短: {} 字节, 至少要有 12 字节 nonce",
                payload.len()
            ));
        }
        let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
        let nonce = <&Nonce>::try_from(nonce).map_err(|_| anyhow!("nonce 长度错误"))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| anyhow!("chacha20poly1305 密钥长度必须是 32 字节"))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("解密失败: 密钥不对或密文已被篡改"))
    }
}

impl KeyLoader for Chacha20 {
    async fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path).await?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Chacha20 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = Key::<ChaCha20Poly1305>::generate();
        let key = key.to_vec();
        Ok(vec![key])
    }
}

impl Blake3 {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = key
            .try_into()
            .map_err(|_| anyhow!("密钥长度错误！期望 32 字节，实际拿到 {} 字节", key.len()))?;

        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl TextSign for Blake3 {
    async fn sign(&self, reader: impl AsyncRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        tokio::pin!(reader);
        reader.read_to_end(&mut buf).await?;

        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    async fn verify(&self, reader: impl AsyncRead, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        tokio::pin!(reader);
        reader.read_to_end(&mut buf).await?;
        let expected = blake3::keyed_hash(&self.key, &buf);
        Ok(expected == *sig)
    }
}

impl KeyLoader for Blake3 {
    async fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path).await?;
        Self::try_new(key.trim_ascii_end())
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = gen_pass::process_genpass(32, false, false, false, false)?;
        let key = key.trim();
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl Ed25519Signer {
    fn new(key: SigningKey) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let signingkey = SigningKey::try_from(key)?;
        let key = Ed25519Signer::new(signingkey);
        Ok(key)
    }
}

impl TextSign for Ed25519Signer {
    async fn sign(&self, reader: impl AsyncRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        tokio::pin!(reader);
        reader.read_to_end(&mut buf).await?;
        let sig = self.key.sign(&buf);

        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyLoader for Ed25519Signer {
    async fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path).await?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut rng = UnwrapErr(SysRng);
        let sk = SigningKey::generate(&mut rng);
        let pk = sk.verifying_key().as_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl Ed25519Verifier {
    fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = VerifyingKey::try_from(key)?;
        let key = Ed25519Verifier::new(key);
        Ok(key)
    }
}

impl TextVerify for Ed25519Verifier {
    async fn verify(&self, reader: impl AsyncRead, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        tokio::pin!(reader);
        reader.read_to_end(&mut buf).await?;
        let sig = Signature::try_from(sig)?;
        Ok(self.key.verify(&buf, &sig).is_ok())
    }
}

impl KeyLoader for Ed25519Verifier {
    async fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path).await?;
        Self::try_new(&key)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const BLAKE3_KEY: &str = "fixtures/blake3.txt";
    const ED25519_SK: &str = "fixtures/ed25519.sk";
    const ED25519_PK: &str = "fixtures/ed25519.pk";
    const CHACHA_KEY: &str = "fixtures/chacha.txt";

    /// 内容永不变化的测试输入。
    const MESSAGE: &str = "fixtures/message.txt";
    const CIPHERTEXT: &str = "fixtures/chacha-ciphertext.txt";

    /// trait 级测试用的内存消息，不碰文件系统
    const MSG: &[u8] = b"the quick brown fox jumps over the lazy dog\n";

    // ════ blake3：签名 / 验签 ══════════════════════════════

    /// 已知答案测试：固定输入 + 固定密钥 → 固定签名。
    /// 它守的是「算法没有被悄悄换掉」—— 升级 blake3 大版本、
    /// 改了 base64 字母表、密钥加载逻辑变了，这条都会红。
    ///
    /// 常量生成方式（只需一次）：
    ///   cargo run -- text sign -i fixtures/message.txt -k fixtures/blake3.txt
    #[tokio::test]
    async fn test_process_text_verify() -> Result<()> {
        const KNOWN_SIG: &str = "i4QSIlqe8y1nqt_vp_Va-4Kr81Yy_-88Ay7vXojp8pM";

        let ret =
            process_text_verify(MESSAGE, BLAKE3_KEY, TextSignFormat::Blake3, KNOWN_SIG).await?;
        assert!(ret, "已知签名应当验证通过");
        Ok(())
    }

    /// 往返：签出来的能验回去
    #[tokio::test]
    async fn test_process_text_sign() -> Result<()> {
        let sig = process_text_sign(MESSAGE, BLAKE3_KEY, TextSignFormat::Blake3).await?;
        let ret = process_text_verify(MESSAGE, BLAKE3_KEY, TextSignFormat::Blake3, &sig).await?;
        assert!(ret);
        Ok(())
    }

    /// 反例：消息被改过必须验不过。
    /// 这条之前完全没有 —— 一个永远返回 true 的 verify 能通过原来的全部测试。
    #[tokio::test]
    async fn test_verify_rejects_tampered_message() -> Result<()> {
        let sig = process_text_sign(MESSAGE, BLAKE3_KEY, TextSignFormat::Blake3).await?;

        // 拿另一份内容配这个签名，必须返回 false（而不是 Err）
        let ret =
            process_text_verify("Cargo.toml", BLAKE3_KEY, TextSignFormat::Blake3, &sig).await?;
        assert!(!ret, "内容不同的消息不应通过验证");
        Ok(())
    }

    /// 反例：换一把密钥必须验不过。
    /// 在 trait 层做，密钥直接给字节，不依赖 fixture 文件。
    #[tokio::test]
    async fn test_blake3_rejects_wrong_key() -> Result<()> {
        let alice = Blake3::try_new([1u8; 32])?;
        let mallory = Blake3::try_new([2u8; 32])?;

        let sig = alice.sign(MSG).await?;
        assert!(alice.verify(MSG, &sig).await?);
        assert!(!mallory.verify(MSG, &sig).await?, "换一把密钥不应通过验证");
        Ok(())
    }

    // ════ chacha20poly1305：加密 / 解密 ═══════════════════

    /// encrypt 走完整路径（读文件 → 加密 → base64），再解回来比对原文
    #[tokio::test]
    async fn test_process_text_encrypt() -> Result<()> {
        let encrypted = process_text_encrypt(MESSAGE, CHACHA_KEY).await?;

        let payload = URL_SAFE_NO_PAD.decode(&encrypted)?;
        let cipher = Chacha20::load(CHACHA_KEY).await?;
        let plaintext = cipher.decrypt(&payload)?;

        assert_eq!(plaintext, fs::read(MESSAGE).await?);
        Ok(())
    }

    /// decrypt 走完整路径，用提交在库里的固定密文。
    /// 这条同时覆盖了「密文文件末尾有换行」的 trim 分支。
    #[tokio::test]
    async fn test_process_text_decrypt() -> Result<()> {
        let plaintext = process_text_decrypt(CIPHERTEXT, CHACHA_KEY).await?;
        let expected = String::from_utf8(fs::read(MESSAGE).await?)?;

        assert_eq!(plaintext, expected);
        Ok(())
    }

    /// nonce 必须每次都换。
    /// 这条挂了说明 nonce 被写死了 —— AEAD 里最严重的错误，
    /// 会让攻击者用 C1 ⊕ C2 = P1 ⊕ P2 直接消掉密钥流。
    #[tokio::test]
    async fn test_chacha_nonce_is_random() -> Result<()> {
        let cipher = Chacha20::try_new([7u8; 32])?;

        let a = cipher.encrypt(MSG).await?;
        let b = cipher.encrypt(MSG).await?;

        assert_ne!(a, b, "同一明文两次加密结果相同 —— nonce 没有随机化");

        // 但两份都要能解回同样的明文
        assert_eq!(cipher.decrypt(&a)?, MSG);
        assert_eq!(cipher.decrypt(&b)?, MSG);
        Ok(())
    }

    /// 篡改密文任意一个 bit，解密必须失败。
    /// 这就是 Poly1305 tag 的全部价值 —— 换成裸 chacha20 这条必挂。
    #[tokio::test]
    async fn test_chacha_rejects_tampered_ciphertext() -> Result<()> {
        let cipher = Chacha20::try_new([7u8; 32])?;
        let mut payload = cipher.encrypt(MSG).await?;

        // 跳过 12 字节 nonce，翻转密文区第一个字节的最低位
        payload[NONCE_LEN] ^= 0x01;

        assert!(
            cipher.decrypt(&payload).is_err(),
            "被篡改的密文必须解密失败"
        );
        Ok(())
    }

    /// 改 nonce 同样要失败 —— nonce 也在认证范围内
    #[tokio::test]
    async fn test_chacha_rejects_tampered_nonce() -> Result<()> {
        let cipher = Chacha20::try_new([7u8; 32])?;
        let mut payload = cipher.encrypt(MSG).await?;

        payload[0] ^= 0x01;

        assert!(cipher.decrypt(&payload).is_err());
        Ok(())
    }

    /// 换一把密钥解不开
    #[tokio::test]
    async fn test_chacha_rejects_wrong_key() -> Result<()> {
        let alice = Chacha20::try_new([7u8; 32])?;
        let bob = Chacha20::try_new([8u8; 32])?;

        let payload = alice.encrypt(MSG).await?;
        assert!(bob.decrypt(&payload).is_err(), "换一把密钥不应解开");
        Ok(())
    }

    /// 输入短于 nonce 长度时给清晰错误，而不是切片越界 panic。
    /// 覆盖 decrypt 里那个 `payload.len() < NONCE_LEN` 分支。
    #[test]
    fn test_chacha_rejects_short_payload() -> Result<()> {
        let cipher = Chacha20::try_new([7u8; 32])?;
        assert!(cipher.decrypt(b"short").is_err());
        Ok(())
    }

    // ════ ed25519（顺带补上，之前完全没测）══════════════

    #[tokio::test]
    async fn test_ed25519_sign_and_verify() -> Result<()> {
        let sig = process_text_sign(MESSAGE, ED25519_SK, TextSignFormat::Ed25519).await?;
        let ret = process_text_verify(MESSAGE, ED25519_PK, TextSignFormat::Ed25519, &sig).await?;
        assert!(ret);
        Ok(())
    }

    #[tokio::test]
    async fn test_ed25519_rejects_tampered_message() -> Result<()> {
        let sig = process_text_sign(MESSAGE, ED25519_SK, TextSignFormat::Ed25519).await?;
        let ret =
            process_text_verify("Cargo.toml", ED25519_PK, TextSignFormat::Ed25519, &sig).await?;
        assert!(!ret);
        Ok(())
    }
}
