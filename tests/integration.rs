//! 集成测试：以外部使用者的视角，跑真实的二进制。
//!
//! 这一层测的是**接线**——哪个子命令连到哪个 process 函数、参数有没有传串。
//! 单元测试覆盖不到它：`process_encode` 自己的单测全绿，也挡不住 CLI 把
//! encode 接到了 decode 上（这个仓库真发生过）。

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn rcli() -> Command {
    Command::cargo_bin("rcli").expect("二进制应当已构建")
}

/// 生成一对 ed25519 密钥，返回持有它们的临时目录（drop 时自动清理）。
fn keypair() -> TempDir {
    let dir = TempDir::new().expect("创建临时目录");
    rcli()
        .args(["text", "generate", "--format", "ed25519", "-o"])
        .arg(dir.path())
        .assert()
        .success();
    dir
}

fn sign_token(dir: &TempDir) -> String {
    let out = rcli()
        .args(["jwt", "sign", "--key"])
        .arg(dir.path().join("ed25519.sk"))
        .args(["--sub", "acme", "--aud", "device1", "--exp", "14d"])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone())
        .expect("token 是 UTF-8")
        .trim()
        .to_owned()
}

#[test]
fn jwt_sign_then_verify_roundtrip() {
    let dir = keypair();
    let token = sign_token(&dir);
    assert_eq!(token.split('.').count(), 3, "JWT 应当是三段");

    rcli()
        .args(["jwt", "verify", "--key"])
        .arg(dir.path().join("ed25519.pk"))
        .args(["-t", &token])
        .assert()
        .success()
        .stdout(contains("\"sub\": \"acme\""))
        .stdout(contains("\"aud\": \"device1\""));
}

#[test]
fn jwt_verify_accepts_matching_audience() {
    let dir = keypair();
    let token = sign_token(&dir);

    rcli()
        .args(["jwt", "verify", "--key"])
        .arg(dir.path().join("ed25519.pk"))
        .args(["-t", &token, "--aud", "device1"])
        .assert()
        .success();
}

#[test]
fn jwt_verify_rejects_wrong_audience() {
    let dir = keypair();
    let token = sign_token(&dir);

    rcli()
        .args(["jwt", "verify", "--key"])
        .arg(dir.path().join("ed25519.pk"))
        .args(["-t", &token, "--aud", "somebody-else"])
        .assert()
        .failure();
}

#[test]
fn jwt_verify_rejects_tampered_signature() {
    let dir = keypair();
    let token = sign_token(&dir);

    let mut tampered = token.clone();
    let last = tampered.pop().expect("token 非空");
    tampered.push(if last == 'A' { 'B' } else { 'A' });

    rcli()
        .args(["jwt", "verify", "--key"])
        .arg(dir.path().join("ed25519.pk"))
        .args(["-t", &tampered])
        .assert()
        .failure();
}

#[test]
fn jwt_verify_rejects_token_signed_by_another_key() {
    let mine = keypair();
    let theirs = keypair();
    let token = sign_token(&theirs);

    rcli()
        .args(["jwt", "verify", "--key"])
        .arg(mine.path().join("ed25519.pk"))
        .args(["-t", &token])
        .assert()
        .failure();
}

#[test]
fn jwt_pubkey_emits_spki_pem() {
    let dir = keypair();

    rcli()
        .args(["jwt", "pubkey", "--key"])
        .arg(dir.path().join("ed25519.pk"))
        .assert()
        .success()
        .stdout(contains("-----BEGIN PUBLIC KEY-----"))
        .stdout(contains("-----END PUBLIC KEY-----"));
}

#[test]
fn jwt_sign_rejects_zero_expiration() {
    let dir = keypair();

    rcli()
        .args(["jwt", "sign", "--key"])
        .arg(dir.path().join("ed25519.sk"))
        .args(["--sub", "a", "--aud", "b", "--exp", "0s"])
        .assert()
        .failure()
        .stderr(contains("greater than zero"));
}

/// 这条不属于 jwt，是补上会话开头那个 bug 的窟窿：
/// `base64 encode` 一度被接到了 `process_decode` 上，单测全绿也没拦住。
#[test]
fn base64_encode_then_decode_roundtrip() {
    let encoded = rcli()
        .args(["base64", "encode", "-i", "-"])
        .write_stdin("hello")
        .assert()
        .success();
    let encoded = String::from_utf8(encoded.get_output().stdout.clone())
        .expect("UTF-8")
        .trim()
        .to_owned();
    assert_eq!(encoded, "aGVsbG8=", "encode 必须真的在编码");

    rcli()
        .args(["base64", "decode", "-i", "-"])
        .write_stdin(encoded)
        .assert()
        .success()
        .stdout(contains("hello"));
}
