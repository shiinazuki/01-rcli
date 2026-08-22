# rcli 命令速查

下面每一条都在本机实测跑过。

> `cargo run` 后面的 `--` **不能省** —— 它把参数交给程序，不加的话 cargo 会自己吞掉
> `-i` / `--format` 这类 flag。
>
> 嫌每次编译慢就先装一次 `cargo install --path .`，之后把所有 `cargo run --` 换成 `rcli`。

**依赖的示例文件**

| 路径 | 用途 |
|---|---|
| `assets/juventus.csv` | csv 示例数据 |
| `fixtures/message.txt` | text 签名 / 加密的示例明文 |
| `fixtures/ed25519.sk` `.pk` | ed25519 密钥对（`text generate` 生成） |
| `fixtures/blake3.txt` | blake3 密钥 |
| `fixtures/chacha.txt` | chacha20 密钥 |

---

## 0. 帮助

```bash
cargo run -- --help
cargo run -- --version
cargo run -- csv --help
cargo run -- base64 encode --help
cargo run -- text --help
cargo run -- http --help
cargo run -- jwt sign --help
```

---

## 1. `csv` — CSV 转 JSON / YAML / TOML

```bash
# 默认转 json，输出到 output.json
cargo run -- csv -i assets/juventus.csv

# 指定格式与输出文件
cargo run -- csv -i assets/juventus.csv --format json -o players.json
cargo run -- csv -i assets/juventus.csv --format yaml -o players.yaml
cargo run -- csv -i assets/juventus.csv --format toml -o players.toml

# 自定义分隔符 / 无表头
cargo run -- csv -i assets/juventus.csv -d ';' -o out.json
cargo run -- csv -i assets/juventus.csv --no-header -o out.json
```

| 参数 | 默认 | 说明 |
|---|---|---|
| `-i, --input` | 必填 | 输入 CSV |
| `-o, --output` | `output.<格式>` | 输出文件 |
| `--format` | `json` | `json` / `yaml` / `toml` |
| `-d, --delimiter` | `,` | 分隔符 |
| `--no-header` | 关 | 首行不是表头 |

---

## 2. `genpass` — 随机密码

```bash
# 默认 16 位，全字符集；zxcvbn 强度评分打到 stderr
cargo run -- genpass

cargo run -- genpass -l 32
cargo run -- genpass -l 20 --no-symbol
cargo run -- genpass -l 24 --no-uppercase --no-symbol

# 只要密码本身，丢掉强度评分
cargo run -- genpass -l 32 2>/dev/null
```

`-l` 最小值 8。四个 `--no-*` 开关分别关掉大写 / 小写 / 数字 / 符号。

---

## 3. `base64` — 编解码

```bash
# -i 默认是 -（标准输入）
echo -n 'hello' | cargo run -- base64 encode -i -          # aGVsbG8=
echo -n 'aGVsbG8=' | cargo run -- base64 decode -i -       # hello

# 往返
echo -n 'hello' | cargo run -- base64 encode -i - | cargo run -- base64 decode -i -

# 从文件
cargo run -- base64 encode -i Cargo.toml
cargo run -- base64 decode -i fixtures/b64.txt

# URL-safe 字母表
echo -n 'hello' | cargo run -- base64 encode -i - --format url-safe
```

`--format` 可选 `standard`（默认）/ `url-safe`。

---

## 4. `text` — 签名 / 验签 / 加解密 / 生成密钥

### 生成密钥

`-o` 指向一个**已存在的目录**，不是文件。

```bash
cargo run -- text generate --format blake3   -o fixtures/   # -> blake3.txt
cargo run -- text generate --format ed25519  -o fixtures/   # -> ed25519.sk + ed25519.pk
cargo run -- text generate --format chacha20 -o fixtures/   # -> chacha.txt
```

### blake3 签名 / 验签（对称，同一把密钥）

```bash
cargo run -- text sign -i fixtures/message.txt -k fixtures/blake3.txt --format blake3
```

把上一步输出的签名填进 `--sig`，返回 `true` / `false`：

```bash
cargo run -- text verify -i fixtures/message.txt -k fixtures/blake3.txt --format blake3 --sig <签名>
```

### ed25519 签名 / 验签（非对称，私钥签、公钥验）

```bash
cargo run -- text sign -i fixtures/message.txt -k fixtures/ed25519.sk --format ed25519
cargo run -- text verify -i fixtures/message.txt -k fixtures/ed25519.pk --format ed25519 --sig <签名>
```

### chacha20-poly1305 加解密

```bash
cargo run -- text encrypt -i fixtures/message.txt -k fixtures/chacha.txt

# 密文喂给 decrypt（-i - 读标准输入）
echo -n '<密文>' | cargo run -- text decrypt -i - -k fixtures/chacha.txt
```

一行搞定加密再解密：

```bash
cargo run -- text encrypt -i fixtures/message.txt -k fixtures/chacha.txt | cargo run -- text decrypt -i - -k fixtures/chacha.txt
```

---

## 5. `http` — 静态文件服务 / 批量生成 index.html

```bash
# 当前目录起服务，默认 8080
cargo run -- http serve

cargo run -- http serve --dir fixtures --port 8080
cargo run -- http serve --dir . --port 3000

# 长时间跑建议 release，debug 版传大文件明显慢
cargo run --release -- http serve --dir . --port 8080
```

```bash
# 为目录树里每个子目录生成 index.html（已存在的会跳过）
cargo run -- http index --dir fixtures

# 强制覆盖已有的 index.html
cargo run -- http index --dir fixtures --force
```

---

## 6. `jwt` — 签发 / 验证 JWT（EdDSA / Ed25519）

### 准备密钥（只需一次）

```bash
cargo run -- text generate --format ed25519 -o fixtures/
```

### 签发

```bash
cargo run -- jwt sign --key fixtures/ed25519.sk --sub acme --aud device1 --exp 14d
cargo run -- jwt sign --key fixtures/ed25519.sk --sub alice --aud web --exp 1h
```

`--exp` 走 humantime 写法：`14d` / `1h` / `30m` / `2w` / `90s`，默认 `14d`，**不接受 0**。

### 验证

```bash
cargo run -- jwt verify --key fixtures/ed25519.pk -t <token>

# 带受众校验：--aud 对不上会失败
cargo run -- jwt verify --key fixtures/ed25519.pk -t <token> --aud device1
```

### 一条龙：签发后立刻验证

```bash
cargo run -- jwt sign --key fixtures/ed25519.sk --sub acme --aud device1 --exp 14d > /tmp/tok.txt
```

```bash
cargo run -- jwt verify --key fixtures/ed25519.pk -t "$(cat /tmp/tok.txt)"
```

### 在 jwt.io 上验证

```bash
cargo run -- jwt pubkey --key fixtures/ed25519.pk
```

1. `jwt sign` 拿到的 token 贴到 jwt.io 左边
2. 页面 algorithm 选 **EdDSA**
3. 上面这条命令输出的整段 PEM（含 `BEGIN` / `END` 两行）贴到 public key 框
4. 应显示 **Signature Verified**

想本地先确认 PEM 合法：

```bash
cargo run -- jwt pubkey --key fixtures/ed25519.pk | openssl pkey -pubin -text -noout
```

---

## 7. 开发 / CI

**不要直接敲 `cargo fmt`** —— `rustfmt.toml` 里有 10 条是 unstable 选项，stable 的 rustfmt
只会打一串 Warning 然后静默忽略，退出码还是 0，看起来像成功了。

```bash
just fmt        # 走 nightly rustfmt + taplo
just ci         # 本地跑一遍 CI 全套
just audit      # cargo-deny 四关：advisories / bans / licenses / sources
just hack       # feature powerset 检查
```

不用 just 的等价命令：

```bash
cargo +nightly fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check
```

---

## 8. 容易踩的地方

- `cargo run` 后面的 `--` 不能省，否则 flag 被 cargo 自己吃掉。
- `text generate` 的 `-o` 指的是**目录**，且必须已经存在。
- `fixtures/` 里是私钥，别提交进 git。
- jwt 的 `--key`：`sign` 用 `.sk`（私钥），`verify` 和 `pubkey` 用 `.pk`（公钥）。传反了报 `InvalidEddsaKey`。
- jwt 验签默认容忍 **60 秒**时钟偏差（jsonwebtoken 的 `Validation.leeway`），所以刚过期一小会儿的 token 仍然验得过。
- 本机设了全局共享 target 目录 `/Users/shiina/.target`。同名 crate 的二进制会互相覆盖 —— 同时开着两份 rcli（比如另一个 worktree）时，`cargo run` 可能跑到另一份的产物，且没有任何提示。遇到「改了代码没生效」先想到这条，可用 `CARGO_TARGET_DIR=/tmp/xxx cargo run ...` 隔离验证。
