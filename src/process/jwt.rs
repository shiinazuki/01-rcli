//! 只支持 `EdDSA` 的 `jsonwebtoken` 加密后端。
//!
//! `jsonwebtoken` 11 沿用了 rustls 那套 `CryptoProvider`：库本身不带任何加密实现，
//! 必须显式装一个后端，否则**编译能过、一跑就 panic**。
//!
//! 官方两个后端都有代价：`rust_crypto` 会拖进 `rsa`（RUSTSEC-2023-0071，Marvin
//! 时序侧信道，至今无补丁），`aws_lc_rs` 要 C 工具链、还 vendor 了七十多个 perl/sh
//! 脚本会撞 deny.toml 的 `interpreted = "deny"`。我们只用 `EdDSA`，自己实现一个最省事，
//! 直接复用项目里已有的 ed25519-dalek，两样都不引入。
//!
//! 附带的好处：密钥字节的含义由这里说了算，所以 `text generate --format ed25519`
//! 产出的**裸 32 字节**可以直接喂进来，不必先转成 PKCS#8 DER。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use ed25519_dalek::{
    Signature, SigningKey, VerifyingKey,
    pkcs8::{EncodePublicKey, spki::der::pem::LineEnding},
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
    crypto::{CryptoProvider, JwtSigner, JwtVerifier, KeyUtils},
    errors::{Error as JwtError, ErrorKind},
    signature::{Error as SigError, Signer, Verifier},
};
use serde::{Deserialize, Serialize};
use tokio::fs;

/// 把 ed25519-dalek 的签名包装成 `jsonwebtoken` 要的形状。
#[derive(Debug)]
struct Edsigner(SigningKey);

#[derive(Debug)]
struct EdVerifier(VerifyingKey);

impl Signer<Vec<u8>> for Edsigner {
    fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>, SigError> {
        Ok(ed25519_dalek::Signer::sign(&self.0, msg)
            .to_bytes()
            .to_vec())
    }
}

impl Verifier<Vec<u8>> for EdVerifier {
    fn verify(&self, msg: &[u8], sig: &Vec<u8>) -> Result<(), SigError> {
        let sig = Signature::from_slice(sig).map_err(|_| SigError::new())?;

        self.0.verify_strict(msg, &sig).map_err(|_| SigError::new())
    }
}

impl JwtSigner for Edsigner {
    fn algorithm(&self) -> Algorithm {
        Algorithm::EdDSA
    }
}

impl JwtVerifier for EdVerifier {
    fn algorithm(&self) -> Algorithm {
        Algorithm::EdDSA
    }
}

/// 密钥文件必须正好 32 字节，这是 ed25519 的固定长度。
fn raw32(bytes: &[u8]) -> Result<[u8; 32], JwtError> {
    bytes
        .try_into()
        .map_err(|_| JwtError::from(ErrorKind::InvalidEddsaKey))
}

static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: |alg, key| {
        if *alg != Algorithm::EdDSA {
            return Err(ErrorKind::InvalidAlgorithm.into());
        }
        Ok(Box::new(Edsigner(SigningKey::from_bytes(&raw32(
            key.as_bytes(),
        )?))))
    },
    verifier_factory: |alg, key| {
        if *alg != Algorithm::EdDSA {
            return Err(ErrorKind::InvalidAlgorithm.into());
        }
        let vk = VerifyingKey::from_bytes(&raw32(key.try_get_as_bytes()?)?)
            .map_err(|_| JwtError::from(ErrorKind::InvalidEddsaKey))?;
        Ok(Box::new(EdVerifier(vk)))
    },
    key_utils: KeyUtils::new_unimplemented(),
};

/// 安装本项目的 `EdDSA` 后端。进程内只有第一次调用真正生效，重复调用是安全的空操作。
pub(super) fn install_crypto_provider() {
    let _ = PROVIDER.install_default();
}

/// JWT 载荷。字段名就是 JWT 规范里的 claim 名，`serde` 直接照着序列化。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// subject：这个 token 代表谁
    pub sub: String,
    /// audience：这个 token 打算给谁用
    pub aud: String,
    /// expiration time，Unix 秒
    pub exp: u64,
    /// issued at，Unix 秒
    pub iat: u64,
}
/// 用 ed25519 私钥签出一个 `EdDSA` 的 JWT。
///
/// `key` 是 `text generate --format ed25519` 产出的 `ed25519.sk`，裸 32 字节。
///
/// # Errors
///
/// 密钥读不出来、长度不对，或者系统时钟早于 Unix 纪元时返回错误。
pub async fn process_jwt_sign(key: &str, sub: &str, aud: &str, exp: Duration) -> Result<String> {
    install_crypto_provider();

    let signing_hey = fs::read(key).await?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        sub: sub.to_owned(),
        aud: aud.to_owned(),
        exp: now.saturating_add(exp.as_secs()),
        iat: now,
    };
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::EdDSA),
        &claims,
        &EncodingKey::from_ed_der(&signing_hey),
    )?;

    Ok(token)
}

/// 用 ed25519 公钥验一个 `EdDSA` 的 JWT，通过就把载荷还回来。
///
/// `key` 是 `ed25519.pk`，裸 32 字节。`aud` 给了就校验受众，不给就跳过这一项
/// ——注意不能什么都不做：token 里带 `aud` 而 `Validation` 不表态时，
/// `jsonwebtoken` 会直接判 `InvalidAudience`。
///
/// # Errors
///
/// 公钥读不出来、签名不对、token 过期、或者 `aud` 对不上时返回错误。
pub async fn process_jwt_verify(key: &str, token: &str, aud: Option<&str>) -> Result<Claims> {
    install_crypto_provider();

    let verifying_key = fs::read(key).await?;
    let mut validation = Validation::new(Algorithm::EdDSA);
    match aud {
        Some(aud) => validation.set_audience(&[aud]),
        None => validation.validate_aud = false,
    }

    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_ed_der(&verifying_key),
        &validation,
    )?;
    Ok(data.claims)
}

/// 把裸 32 字节的 ed25519 公钥导出成 SPKI PEM。
///
/// jwt.io 验 `EdDSA` 时要贴的就是这个格式，而 `text generate` 存的是裸字节，
/// 中间差一层 ASN.1 封装。
///
/// # Errors
///
/// 公钥读不出来、长度不对、或者不是合法的曲线点时返回错误。
pub async fn process_jwt_pubkey(key: &str) -> Result<String> {
    let bytes = fs::read(key).await?;
    let bytes: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("public key must be exactly 32 bytes, got {}", bytes.len()))?;
    let pem = VerifyingKey::from_bytes(&bytes)?.to_public_key_pem(LineEnding::LF)?;
    Ok(pem)
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Probe {
        sub: String,
        exp: u64,
    }

    /// 固定种子，测试要可复现
    fn keypair() -> ([u8; 32], [u8; 32]) {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        (sk.to_bytes(), *sk.verifying_key().as_bytes())
    }

    fn sign_with(sk: &[u8; 32]) -> String {
        encode(
            &Header::new(Algorithm::EdDSA),
            &Probe {
                sub: "acme".into(),
                exp: 4_102_444_800, // 2100 年，测试期间不会过期
            },
            &EncodingKey::from_ed_der(sk),
        )
        .expect("签名应当成功")
    }

    fn validation() -> Validation {
        let mut v = Validation::new(Algorithm::EdDSA);
        v.validate_aud = false;
        v
    }
    #[test]
    fn roundtrip_sign_then_verify() {
        install_crypto_provider();
        let (sk, pk) = keypair();
        let token = sign_with(&sk);

        let decoded = decode::<Probe>(&token, &DecodingKey::from_ed_der(&pk), &validation())
            .expect("验签应当成功");
        assert_eq!(decoded.claims.sub, "acme");
    }

    #[test]
    fn verify_rejects_tampered_token() {
        install_crypto_provider();
        let (sk, pk) = keypair();
        let mut tampered = sign_with(&sk);
        let last = tampered.pop().expect("token 非空");
        tampered.push(if last == 'A' { 'B' } else { 'A' });

        assert!(decode::<Probe>(&tampered, &DecodingKey::from_ed_der(&pk), &validation()).is_err());
    }

    #[test]
    fn verify_rejects_wrong_public_key() {
        install_crypto_provider();
        let (sk, _) = keypair();
        let other = *SigningKey::from_bytes(&[9u8; 32])
            .verifying_key()
            .as_bytes();
        let token = sign_with(&sk);

        assert!(decode::<Probe>(&token, &DecodingKey::from_ed_der(&other), &validation()).is_err());
    }
}
