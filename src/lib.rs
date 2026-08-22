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

#[macro_use]
mod macros;
mod cli;
mod process;
mod utils;

use anyhow::Result;
pub use cli::{Base64Format, Opts, OutputFormat, TextKeyFormat, TextSignFormat};
pub use process::{
    Claims, process_csv, process_decode, process_encode, process_genpass, process_http_index,
    process_http_serve, process_jwt_pubkey, process_jwt_sign, process_jwt_verify,
    process_text_decrypt, process_text_encrypt, process_text_generate, process_text_sign,
    process_text_verify,
};
pub use utils::{InputReader, get_reader, write_secret};

pub(crate) trait CmdExecutor {
    async fn execute(self) -> Result<()>;
}

pub async fn parse_cmd(opts: Opts) -> Result<()> {
    opts.cmd.execute().await
}
