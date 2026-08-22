# rcli

[![build](https://github.com/shiinazuki/rcli/actions/workflows/build.yaml/badge.svg)](https://github.com/shiinazuki/rcli/actions/workflows/build.yaml)
[![audit](https://github.com/shiinazuki/rcli/actions/workflows/audit.yaml/badge.svg)](https://github.com/shiinazuki/rcli/actions/workflows/audit.yaml)
![license](https://img.shields.io/badge/license-MIT-blue)

RUST CLI

## 快速开始

```bash
git add -A && git commit -m "chore: 从模板初始化项目"   # 生成器只做了 git init

just doctor          # 体检：工具链组件与配套工具是否齐全（会告诉你缺什么、怎么装）
just install-tools   # 安装配套 cargo 工具（首次）
just bootstrap       # 生成 Cargo.lock + 安装 git 钩子（首次）
just ci              # 跑一遍完整检查，确认环境就绪
just dev             # 开始写代码：bacon 盯着文件变化实时重跑 clippy
```

`just` 不带参数会列出全部命令（按用途分组）。

## 项目骨架
```
src/
  lib.rs           业务逻辑都写在这一侧
  main.rs          可执行入口：初始化日志、错误收口
tests/
  integration.rs   集成测试：以外部使用者的视角调用 lib 的公开 API
```

**为什么二进制项目也有 `lib.rs`**：`main.rs` 里的东西集成测试（`tests/` 是独立
crate）、benchmark、doctest 都够不着，逻辑留在那边就只能靠手工跑一遍程序来验证。
把 `main.rs` 保持薄薄一层、逻辑放进 `lib.rs`，这三样立刻都能用上。

所以 `main.rs` 长起来了，就说明有东西该往 `lib.rs` 挪了。

## 开发环境

### Rust 工具链

工具链由 [`rust-toolchain.toml`](rust-toolchain.toml) 固定为 **stable**，首次进入目录时 rustup 会自动安装。
注意这个文件会**覆盖你 rustup 的全局默认工具链**，在本项目目录内一律以它为准。

同时会装上 `rustfmt`、`clippy`、`rust-src`（rust-analyzer 解析标准库要用，
缺了它编辑器里对 `std` 没有补全和跳转）和 `llvm-tools-preview`（覆盖率要用；
`llvm-tools` 是它现在的规范名，两者装的是同一份东西）。
`rust-analyzer`、`miri`、交叉编译 target 等可选项在该文件里以注释列出，按需打开。

[`rustfmt.toml`](rustfmt.toml) 用到了 `imports_granularity`、`group_imports`、`wrap_comments`
等 unstable 选项，只有 nightly 的 rustfmt 才认（stable 会**静默忽略**它们），
所以格式化请一律走 `just fmt` / `just lint`，不要手写 `cargo fmt`。

本项目跑在 stable 上，需要额外装一次 nightly 的 rustfmt——`just install-tools` 会替你装，
`just doctor` 会检查它在不在。手工装的话：

```bash
rustup toolchain install nightly --allow-downgrade --profile minimal --component rustfmt
```

### 编译器自己崩了（ICE）怎么办

`error: internal compiler error` 加上工作目录里冒出来的 `rustc-ice-*.txt`，说明是
**编译器崩了**，不是你的代码有语法或类型错误。别急着改自己的代码，先跑：

```bash
just ice
```

它会从那堆几百行的栈回溯里摘出三样真正有用的东西：panic 消息、**产生它的编译器版本**、
以及崩溃时的 query stack。

其中最容易被跳过、却最关键的是版本那一行。把它和 `rust-toolchain.toml` 里的 `channel` 对一下：

- **对不上** —— 你的工具链配置根本没生效（多半是有人跑过 `rustup override set`，
  它的优先级比 `rust-toolchain.toml` 高且完全静默）。先跑 `just doctor`，它会直接点出来。
- **对得上** —— 就是这一版编译器在你的代码上崩了。query stack 会指向具体是在编译哪个
  函数 / 类型，那里通常有个换个写法就能绕开的构造。要立刻恢复工作的话，把 `channel`
  钉到前几天的 nightly（`nightly-YYYY-MM-DD`），修好之后再挪回来。

⚠️ `.gitignore` 已经挡住了 `rustc-ice-*.txt`，所以 **`git status` 干净不代表没有转储**
——用 `just ice` 看，别看 git。

### MSRV

`Cargo.toml` 里的 `rust-version` 声明了最低支持版本。它**只是下限，不限制上限**，
用更新的 stable 或 nightly 编译都没问题。

⚠️ 它默认是 `1.88`，而**不是** edition 2024 的地板值 1.85。原因是把 MSRV 定低有一条
静默的安全代价：配上 `resolver = "3"` 与 `.cargo/config.toml` 的
`incompatible-rust-versions = "fallback"`，某个依赖的新版本一旦把自己的 `rust-version`
抬到你的 MSRV 以上，resolver 会**一声不吭地退回旧版本**——而安全补丁往往就在新版本里。

真实例子：`time` 的 RUSTSEC-2026-0009 补在 0.3.47，那个版本要求 rustc 1.88。

| `rust-version` | resolver 选中的 `time` | `just audit` |
| --- | --- | --- |
| `1.85` | 0.3.45（有漏洞） | FAILED |
| `1.88` | 0.3.55 | 通过 |

两种情况下 `cargo build` 都一路绿灯，唯一能发现它的是 `just audit`。

要支持更老的 rustc 就往下调，但那等于把上面这条风险重新打开；再撞上同类问题时，
优先抬 `rust-version`，而不是在 `deny.toml` 里 ignore 掉告警。

`just msrv` 和 CI 的 msrv job 会真的用那个版本编译一遍来验证声明属实。

### 配套工具

先装 [just](https://github.com/casey/just)（命令入口，见 [`justfile`](justfile)），
再让它把剩下的装齐：

```bash
cargo install just
just install-tools
just doctor          # 确认真的都装上了
```

`install-tools` 会优先用 [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)
直接下载上游发布的预编译二进制——这些工具从源码编译一遍要十几分钟，binstall 只要几十秒。
强烈建议先装上它：

```bash
cargo install cargo-binstall
```

装的是这些（也可以按需逐个 `cargo install --locked <名字>`）：

| 工具 | 用途 |
| --- | --- |
| `cargo-nextest` | 测试运行器（比 `cargo test` 快，输出也更清楚） |
| `cargo-deny` | 依赖安全公告与 License 检查 |
| `cargo-llvm-cov` | 覆盖率 |
| `cargo-release` | 发版 |
| `cargo-outdated` | 检查依赖是否有新版本 |
| `cargo-machete` | 找出声明了却没用到的依赖 |
| `cargo-semver-checks` | 公开 API 的破坏性变更检查 |
| `cargo-hack` | feature 幂集检查 |
| `typos-cli` | 拼写检查 |
| `taplo-cli` | TOML 格式化与检查（rustfmt 只管 `.rs`） |
| `git-cliff` | 生成 CHANGELOG |
| `bacon` | 后台实时监控 |

### git 钩子

```bash
just hooks
```

钩子脚本就在仓库的 [`.githooks/`](.githooks/) 里，`just hooks` 把 `core.hooksPath`
指过去（git 不会自动信任仓库里的钩子，所以每个 clone 都要跑一次）：

| 钩子 | 作用 | 大概耗时 |
| --- | --- | --- |
| `pre-commit` | 按**本次改动的文件类型**跑快速检查：`.rs` → rustfmt + clippy；`.toml` → taplo；`Cargo.toml` / `Cargo.lock` / `deny.toml` → cargo-deny；外加拼写与私钥检测 | 秒级 |
| `commit-msg` | 校验 Conventional Commits —— CHANGELOG 分组与 cargo-release 的版本推导都依赖它 | 瞬间 |
| `pre-push` | 跑一遍 `just ci`（lint / test / audit） | 十几秒起 |

三层是有意分开的，越往后越全也越慢：

- **`pre-commit` 里刻意不跑测试。** 提交是本地动作，做到一半的活儿也该能存档；
  每次 commit 都等一遍全量测试，最后只会让人养成 `--no-verify` 的习惯——那才是真的失去防线。
- **`cargo deny` 只在依赖真的可能变了时才跑**（`Cargo.toml` / `Cargo.lock` / `deny.toml`
  被改动）。它要解析整棵依赖树，改一行注释也跑一遍纯属浪费。
- **`pre-push` 才是真正的闸门。** commit 是本地的、随时能 `amend` / `rebase` 改掉；
  push 才是「出去了」。所以全量检查放在这一层，没过就推不出去。

> `pre-commit` 检查的是**工作区**当前状态，不是暂存区快照。`git commit -a` 下两者一致；
> 用 `git add -p` 做部分暂存时，未暂存的改动也会被算进来。要精确只检查暂存内容，
> 得先 stash 掉未暂存的改动再恢复——那是个经典的丢代码来源，模板刻意不做。

临时跳过：`git commit --no-verify` / `git push --no-verify`。
停用：`git config --unset core.hooksPath`。

> **从旧版模板升级过来的项目**：早期版本用的是 pre-commit，`.git/hooks/` 里可能还留着
> 它装的脚本。`.pre-commit-config.yaml` 已经不在了，那些脚本于是会在每次 `git commit`
> 时报一句 `No .pre-commit-config.yaml file was found`。跑一次 `just hooks` 即可——
> 它会把这类残留自动清掉。

### 容器里开发（可选）

[`.devcontainer/`](.devcontainer/) 里有一份 Dev Container 配置，
VS Code 的 Dev Containers 插件或 GitHub Codespaces 可以直接用，
省掉本机装工具链的过程。

## 常用命令

`just` 直接列出全部命令（按用途分组）。

```bash
just                    # 列出所有命令
just doctor             # 环境体检
just bootstrap          # 首次拉起：生成 Cargo.lock + 安装 git 钩子
just check              # 快速检查编译
just run -- --help      # 运行程序（仅 bin 项目），-- 后的参数透传给程序
just fmt                # 格式化 .rs（nightly rustfmt）与 .toml（taplo）
just fix                # clippy --fix 自动修复 + 格式化
just dev                # bacon 实时监控
just doc                # 生成并打开 API 文档
just bench              # 跑 benchmark
just flamegraph         # 采样生成火焰图（仅 bin 项目）
just clean              # 清理编译产物与本地报告

just lint               # 格式化检查（.rs + .toml）+ clippy + typos + 文档警告
just test               # 运行测试（含 doctest）
just coverage           # 生成覆盖率报告 lcov.info
just coverage-html      # HTML 覆盖率报告并打开
just audit              # cargo deny check
just hack               # feature 幂集检查
just msrv               # 验证 MSRV 能编译（nightly 项目自动转去跑 nll）
just nll                # 用 stable 的借用检查器编一遍（仅 nightly 项目有意义）
just ice                # 解读 rustc-ice-*.txt（编译器自己崩了的时候用）
just ci                 # 本地跑一遍 CI 的主要检查（lint / test / audit）

just unused             # 找出没用到的依赖（cargo-machete）
just semver             # 公开 API 破坏性变更检查（仅纯库项目）

just update             # 升级 Cargo.lock 并重新审计
just outdated           # 列出可升级的依赖

just changelog          # 刷新 CHANGELOG.md
just release minor      # 发版预演：跑全套检查 + 干跑一遍，不改动任何东西
just release-execute minor  # 真正发版：抬版本号 + CHANGELOG + tag + 推送
                            # （紧接在 release 之后跑，不再重复跑一遍检查）
```

`just ci` 包含 `lint` / `test` / `audit` 三项。其中 `lint` 和 CI 的 lint job 严格对齐，
**含 `cargo doc` 的文档警告检查**——`[workspace.lints.rustdoc]` 里 `bare_urls`、
`invalid_html_tags` 这些都只是 `warn`，本地不跑 `cargo doc` 就看不见，
推上去才会在 CI 的 `RUSTDOCFLAGS="-D warnings"` 上挂掉。

`unused`、`semver`、`hack`、`msrv`、`nll` 刻意留在外面手动跑：第一个对宏里用到的依赖会误报，
第二个需要和已发布版本联网比对，第三个要额外装 cargo-hack、feature 多起来还会变慢，
第四个会 `rustup toolchain install` 往你机器上装一整条工具链，
第五个换了 `RUSTFLAGS` 等于一次全量重编——
都不适合塞进「随手跑一下」的命令里。

除 `unused` 外，它们在两套 CI 里都有对应的 job（`semver` 只对纯库项目生效）。`unused` 刻意**没有**进任何 CI：误报率高的检查一旦当上门禁，
结果只会是所有人都学会忽略它。需要时手动跑 `just unused`。

## 工程结构

`Cargo.toml` 里已经铺好了 workspace 骨架：`[workspace.package]`、
`[workspace.dependencies]`、`[workspace.lints]` 三段是给**将来拆分子 crate** 准备的。
现在只有根 crate 一个成员，它通过 `version.workspace = true` / `[lints] workspace = true`
继承这些配置。等你要拆出 `crates/core`、`crates/cli` 时，
只需在 `members` 里登记，子 crate 里同样写 `.workspace = true` 即可，
版本号与 lint 规则不会各自漂移。

### 编译 profile

| profile | 用途 |
| --- | --- |
| `dev` | 自身代码 O0 保证调试体验；依赖 O2（`[profile.dev.package."*"]`），运行时快一个数量级 |
| `test` | O1，比 O0 跑得快又不用等 O3 的编译时间 |
| `release` | O3 + thin LTO + `codegen-units = 1` + strip |
| `profiling` | 继承 release 但保留符号，火焰图才有可读函数名：`just flamegraph` |
| `bench` | 继承 release 且保留符号，保证 benchmark 测的是优化后的代码 |

`Cargo.toml` 末尾还注释着两项按需打开的配置：`build-override`（加速 proc-macro 编译）
和 `overflow-checks`（release 下也检查整数溢出，账务 / 协议解析类项目建议打开）。

## 项目里的各个配置文件

| 文件 | 作用 |
| --- | --- |
| [`rust-toolchain.toml`](rust-toolchain.toml) | 固定工具链版本与组件 |
| [`rustfmt.toml`](rustfmt.toml) | 格式化规则（含 unstable 选项，走 nightly） |
| [`clippy.toml`](clippy.toml) | Clippy 行为配置（lint 开关在 `Cargo.toml` 的 `[workspace.lints]`） |
| [`deny.toml`](deny.toml) | 依赖的安全公告 / License / 重复版本 / 来源审计，外加 build script 里夹带的二进制与脚本 |
| [`.taplo.toml`](.taplo.toml) | TOML 格式化规则（rustfmt 只管 `.rs`，`.toml` 归 taplo） |
| [`.typos.toml`](.typos.toml) | 拼写检查的词表与排除规则 |
| [`cliff.toml`](cliff.toml) | git-cliff 生成 CHANGELOG 的模板与分组规则 |
| [`release.toml`](release.toml) | cargo-release 的发版流程配置 |
| [`bacon.toml`](bacon.toml) | bacon 实时监控的任务定义 |
| [`justfile`](justfile) | 全部日常命令的入口 |
| [`.config/nextest.toml`](.config/nextest.toml) | 测试运行器配置（含 CI 专用 profile、JUnit、超时与测试分组示例） |
| [`.cargo/config.toml`](.cargo/config.toml) | cargo 项目级配置：网络重试、依赖解析策略，以及链接器 / 并行前端 / 镜像源的开关都收在这里 |
| [`AGENTS.md`](AGENTS.md) | 给 AI 编码助手的项目约定（格式化必须走 nightly、零警告、不许压制 lint 等） |
| [`.githooks/`](.githooks/) | Git 钩子（commit-msg 校验提交信息 / pre-push 跑 `just ci`），`just hooks` 启用 |
| [`.editorconfig`](.editorconfig) | 跨编辑器的基础排版约定 |
| [`.gitattributes`](.gitattributes) | 入库换行统一、二进制标记、`Cargo.lock` 折叠 |
| [`.devcontainer/`](.devcontainer/) | Dev Container / Codespaces 配置 |
| [`.github/workflows/`](.github/workflows/) | CI（build / release / audit） |
| [`.github/dependabot.yml`](.github/dependabot.yml) | 依赖自动升级：cargo / actions |

## CI

推送和 PR 触发 [`build.yaml`](.github/workflows/build.yaml)，并行跑这些 job：

- **detect** —— 探测仓库里有哪些 target，供下面的 job 做条件判断（几秒钟）
- **lint** —— 格式化（`.rs` 走 rustfmt、`.toml` 走 taplo）、拼写、clippy（`-D warnings`）、文档警告
- **test** —— `cargo check` + nextest（CI profile：不 fail-fast、失败重试、输出 JUnit）+ 覆盖率 + doctest
- **deny** —— 依赖的安全公告 / License / 重复版本 / 来源
- **workflows** —— 用 [zizmor](https://docs.zizmor.sh/) 审计 workflow 的**安全性**（脚本注入、
  过宽权限、缓存投毒），再用 [actionlint](https://github.com/rhysd/actionlint) 查**正确性**
  （表达式写错、不存在的 job 依赖、`run:` 里的 shell 语法）——两者不重叠
- **msrv / nll** —— stable 项目：用 `Cargo.toml` 里声明的最低版本编译一遍；
  nightly 项目：改用 `-Zpolonius=off` 编一遍，拦下只有新借用检查器才编得过的代码
- **hack** —— 遍历 feature 幂集，防止「单独开某个 feature 编不过」
- **semver** —— 以上一个 tag 为基线检查公开 API 破坏性变更（仅**纯库**项目，没有 tag 时跳过；
  二进制项目的 `src/lib.rs` 是自用的内部库，不对外承诺 API）
- **docker** —— 构建一次容器镜像确认 Dockerfile 没坏（仅选了 Docker 的项目；只构建不推送）

打 `v*` tag 触发 [`release.yaml`](.github/workflows/release.yaml)：

- **verify** —— 把 tag 指向的 commit 从零验证一遍，并核对 tag 与 `Cargo.toml` 版本一致
- **github-release** —— git-cliff 生成变更说明并创建 Release
- **binaries** —— 五个目标平台（Linux musl x64/arm64、macOS x64/arm64、Windows x64）
  交叉编译、打包、生成 sha256 并挂到 Release 上（仅 bin 项目）。
  再往上一层是**构建来源证明**（SLSA provenance，Sigstore 签名，`gh attestation verify` 可验）：
  校验和只能证明「文件没被改过」，证明回答的是「它是谁造的」——**默认关闭**，
  需要在仓库 Variables 里加 `ATTEST_BUILD_PROVENANCE=true`（私有仓库需要 GitHub Enterprise）
- **crates-io** —— 用 crates.io 的 Trusted Publishing（OIDC，无需长期 token）发布，
  **默认关闭**，需要在仓库 Variables 里加 `PUBLISH_TO_CRATES_IO=true`

[`audit.yaml`](.github/workflows/audit.yaml) 每天定时跑一次依赖审计——
安全公告是「代码没动风险也会变」的东西，只靠 PR 触发发现不了。

两点值得注意：

- **CI 与发布分成两个 workflow**，因为发布流程刻意不使用编译缓存。缓存是可写的，
  一旦发布产物建立在缓存之上，「污染缓存」就等价于「污染 release 二进制」。
- **按 target 裁剪的 job（semver / binaries / docker）一律靠 `detect` 传出的 outputs 判断**，
  而不是在 job 级写 `if: hashFiles(...)`。job 级的 `if:` 在 checkout 之前就求值，
  那时工作区还是空的，hashFiles 恒为空串——条件永远不成立，job 被静默跳过，不报任何错。
- **第三方 action 全部用 commit hash 钉死**（后面的 `# vX.Y.Z` 是给人看的）。
  tag 是可变的，上游账号一旦被攻破，把 `v3` 指向恶意提交就能直接进你的 CI。
  hash 由 dependabot 每周自动更新。

## 提交规范

本项目使用 [Conventional Commits](https://www.conventionalcommits.org/)，
`CHANGELOG.md` 由 [git-cliff](https://git-cliff.org/) 依据提交信息自动生成：

```
feat(parser): 支持嵌套表达式
fix: 修正边界条件下的 panic
docs: 补充 README
```

commit message 由 [`.githooks/commit-msg`](.githooks/commit-msg) 强制校验（`just hooks` 启用后生效）。

## License

协议在生成项目时选定，对应的许可证文件在仓库根目录：
单协议是 `LICENSE`，双协议（MIT OR Apache-2.0）则是 `LICENSE-MIT` 与 `LICENSE-APACHE`。
具体取值见 `Cargo.toml` 的 `license` 字段。
