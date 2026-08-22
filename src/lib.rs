//! `rcli` —— 把几件常用的命令行杂活收在一个二进制里。
//!
//! # 子命令
//!
//! | 命令 | 做什么 |
//! |---|---|
//! | `csv` | 读 CSV，转成 JSON / YAML / TOML |
//! | `genpass` | 生成随机密码，并用 zxcvbn 评估强度 |
//! | `base64` | 标准与 URL-safe 两种字母表的编解码 |
//! | `text` | Blake3 签名验签、Ed25519 签名验签、ChaCha20-Poly1305 加解密 |
//! | `http` | 起一个静态文件服务，或为目录批量生成 index.html |
//!
//! # 结构
//!
//! 分成三层，`main.rs` 只负责 `Opts::parse()` 然后把结果交给 [`parse_cmd`]：
//!
//! - `cli`：clap 的参数定义。除了 [`Opts`] 和几个 `*Format` 枚举，其余类型都是 `pub(crate)` ——
//!   它们是命令行的形状，不是这个库对外承诺的 API。
//! - `process`：真正干活的 `process_*` 函数，每个都不依赖 clap，可以单独调用和测试。
//! - `utils`：读写两侧的共用件，[`get_reader`] 统一处理「文件路径或 `-`（标准输入）」。
//!
//! # 例子
//!
//! 各个 `process_*` 都能脱离命令行单独用：
//!
//! ```
//! # fn main() -> anyhow::Result<()> {
//! let password = rcli::process_genpass(16, false, false, false, false)?;
//! assert_eq!(password.len(), 16);
//! # Ok(())
//! # }
//! ```

mod cli;
mod process;
mod utils;

use anyhow::Result;
pub use cli::{Base64Format, Opts, OutputFormat, TextKeyFormat, TextSignFormat};
use cli::{Base64SubCommand, HttpSubCommand, SubCommand, TextSubCommand};
pub use process::{
    process_csv, process_decode, process_encode, process_genpass, process_http_index,
    process_http_serve, process_text_decrypt, process_text_encrypt, process_text_generate,
    process_text_sign, process_text_verify,
};
use tokio::fs;
pub use utils::{InputReader, get_reader, write_secret};
use zxcvbn::zxcvbn;

pub async fn parse_cmd(opts: Opts) -> Result<()> {
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = &opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(
                &opts.input,
                output,
                opts.format,
                opts.delimiter,
                !opts.no_header,
            )
            .await?;
        }
        SubCommand::GenPass(opts) => {
            let password = process_genpass(
                opts.length,
                opts.no_uppercase,
                opts.no_lowercase,
                opts.no_number,
                opts.no_symbol,
            )?;
            println!("{password}");
            let result = zxcvbn(&password, &[]);
            eprintln!("Password strength: {}", result.score());
        }
        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opts) => {
                let encoded = process_encode(&opts.input, opts.format).await?;
                println!("{encoded}");
            }

            Base64SubCommand::Decode(opts) => {
                let decoded = process_decode(&opts.input, opts.format).await?;
                let decoded = String::from_utf8(decoded)?;
                println!("{decoded}");
            }
        },

        SubCommand::Text(subcmd) => match subcmd {
            TextSubCommand::Sign(opts) => {
                let signed = process_text_sign(&opts.input, &opts.key, opts.format).await?;
                println!("{signed}");
            }

            TextSubCommand::Verify(opts) => {
                let verified =
                    process_text_verify(&opts.input, &opts.key, opts.format, &opts.sig).await?;
                println!("{verified}");
                if !verified {
                    std::process::exit(1);
                }
            }

            TextSubCommand::Generate(opts) => {
                let key = process_text_generate(opts.format)?;
                match opts.format {
                    TextKeyFormat::Blake3 => {
                        write_secret(opts.output.join("blake3.txt"), &key[0]).await?;
                    }
                    TextKeyFormat::Ed25519 => {
                        write_secret(opts.output.join("ed25519.sk"), &key[0]).await?;
                        fs::write(opts.output.join("ed25519.pk"), &key[1]).await?;
                    }
                    TextKeyFormat::Chacha20 => {
                        write_secret(opts.output.join("chacha.txt"), &key[0]).await?;
                    }
                }
            }

            TextSubCommand::Encrypt(opts) => {
                let encrypt = process_text_encrypt(&opts.input, &opts.key).await?;
                println!("{encrypt}");
            }
            TextSubCommand::Decrypt(opts) => {
                let decrypt = process_text_decrypt(&opts.input, &opts.key).await?;
                println!("{decrypt}");
            }
        },

        SubCommand::Http(subcmd) => match subcmd {
            HttpSubCommand::Serve(opts) => {
                process_http_serve(opts.dir, opts.port).await?;
            }

            HttpSubCommand::Index(opts) => {
                let n = process_http_index(opts.dir, opts.force).await?;
                println!("Generated {n} index.html file(s)");
            }
        },
    }
    Ok(())
}
