<!-- JUYA FORK MAINTAINED:BEGIN -->

> [!IMPORTANT]
> **本 fork 的文档边界**：本 README 保留上游的产品介绍和源码构建说明，同时在明确标记的区域补充本 fork 的隐私、分发和协作信息。
>
> 本 fork 特有的维护文档包括：[CHANGELOG.md](CHANGELOG.md)（面向使用者的变更摘要）、[UPSTREAM_TRACE.md](UPSTREAM_TRACE.md)（上游同步事实与决策记录）、[AGENTS.md](AGENTS.md)（维护约定）、[CONTRIBUTING.md](CONTRIBUTING.md)（协作与 issue 规则），以及 [.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/)（issue 模板）。
>
> README 中的 `JUYA FORK MAINTAINED` 区域由本 fork 维护；`UPSTREAM-CARRIED` 区域以同步的上游内容为基础。上游版本和 `SOURCE_REV` 以 [UPSTREAM_TRACE.md](UPSTREAM_TRACE.md) 为准，fork 特有变更则记录在 [CHANGELOG.md](CHANGELOG.md) 中。

> [!NOTE]
> 本构建硬禁用 Claude/Codex/Cursor 兼容、自动会话/工作区制品上传、relay 同步，以及 external OTEL 中的提示词和工具详情字段；聚合元数据遥测、错误报告和反馈路径也在源码中固定关闭。原版 Grok 已有的 `.agents/skills` 与 `.agents/commands` 支持仍保留；OAuth 登录和正常推理不受影响。

> **预编译安装方式可任选**：可以从 [GitHub Releases](https://github.com/juya-ai-lab/grok-build/releases) 下载对应平台二进制，也可以使用 npm/pnpm 全局安装预编译包：
> ```sh
> npm install -g @juya-ai-lab/grok-build
> # 或
> pnpm add -g @juya-ai-lab/grok-build
> ```

<!-- JUYA FORK MAINTAINED:END -->

<!-- UPSTREAM-CARRIED:BEGIN -->

<div align="center">

<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://media.x.ai/v1/website/spacexai-symbol-white-transparent-0c31957f.png">
    <source media="(prefers-color-scheme: light)" srcset="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png">
    <img alt="SpaceXAI logo" src="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png" width="96">
  </picture>
  <br>
  Grok Build (<code>grok</code>)
</h1>

**Grok Build** is SpaceXAI's terminal-based AI coding agent. It runs as a
full-screen TUI that understands your codebase, edits files, executes shell
commands, searches the web, and manages long-running tasks — interactively,
headlessly for scripting/CI, or embedded in editors via the Agent Client
Protocol (ACP).

[Installing a prebuilt release](#installing-a-prebuilt-release) ·
[Changelog](CHANGELOG.md) ·
[Building from source](#building-from-source) ·
[Documentation](#documentation) ·
[Repository layout](#repository-layout) ·
[Development](#development) ·
[Contributing](#contributing) ·
[License](#license)

![Grok Build TUI](https://media.x.ai/v1/website/universe-tui-screenshot-6f7a0837.png)

**Learn more about Grok Build at [x.ai/cli](https://x.ai/cli)**

This repository contains the Rust source for the `grok` CLI/TUI and its agent
runtime. It is synced periodically from the SpaceXAI monorepo.

A small `SOURCE_REV` file at the root records the full monorepo commit SHA
for the version of the code present in this tree.

</div>

<!-- UPSTREAM-CARRIED:END -->

<!-- JUYA FORK MAINTAINED:BEGIN -->

---

## Installing a prebuilt release

预编译安装方式可任选，均不要求本地编译。

### GitHub Releases

从 [GitHub Releases](https://github.com/juya-ai-lab/grok-build/releases) 选择版本，下载与操作系统和 CPU 架构匹配的资产。资产命名格式为：

```text
grok-<version>-<platform>-<arch>[.exe]
```

例如 Linux x86_64、Linux aarch64、macOS x86_64、macOS aarch64，以及 Windows x86_64/aarch64 均有对应资产；同时下载同名 `.sha256` 文件并校验后再运行。下载后将文件命名为 `grok`（Windows 保留 `.exe`）并放入 `PATH`，然后运行：

```sh
grok --version
```

### npm / pnpm

npm/pnpm 包同样提供预编译二进制，并通过 optional dependencies 自动选择当前平台：

```sh
npm install -g @juya-ai-lab/grok-build
# 或
pnpm add -g @juya-ai-lab/grok-build
```

npm 的可用版本以 registry 中已发布的版本为准；GitHub tag 不等同于已发布的 npm 版本。需要固定版本时，在包名后追加已发布版本号，例如：

```sh
npm install -g @juya-ai-lab/grok-build@0.2.118
```

<!-- JUYA FORK MAINTAINED:END -->

<!-- UPSTREAM-CARRIED:BEGIN -->

## Building from source

Requirements:

- **Rust** — the toolchain is pinned by [`rust-toolchain.toml`](rust-toolchain.toml);
  `rustup` installs it automatically on first build.
- **[DotSlash](https://dotslash-cli.com)** — required so hermetic tools under
  [`bin/`](bin/) (notably [`bin/protoc`](bin/protoc)) can download and run.
  Install it and ensure `dotslash` is on your `PATH` **before** building:

  ```sh
  cargo install dotslash
  # or: prebuilt packages — https://dotslash-cli.com/docs/installation/
  /usr/bin/env dotslash --help   # sanity check
  ```

- **protoc** — proto codegen resolves [`bin/protoc`](bin/protoc) via DotSlash,
  or falls back to a `protoc` on `PATH` / `$PROTOC`.
- macOS and Linux are supported build hosts; Windows builds are best-effort
  and not currently tested from this tree.

```sh
cargo run -p xai-grok-pager-bin              # build + launch the TUI
cargo build -p xai-grok-pager-bin --release  # release binary: target/release/xai-grok-pager
cargo check -p xai-grok-pager-bin            # fast validation
```

The binary artifact is named `xai-grok-pager`; official installs ship it as
`grok`. On first launch it opens your browser to authenticate — see the
[authentication guide](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md).

## Documentation

Full online documentation is available at
[docs.x.ai/build/overview](https://docs.x.ai/build/overview).

The user guide ships with the pager crate:
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
— getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition-root package; builds the `xai-grok-pager` binary |
| `crates/codegen/xai-grok-pager` | The TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless entry points |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, file edit, search, ...) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | The rest of the CLI crate closure (config, MCP, markdown, sandbox, ...) |
| `crates/common/`, `crates/build/`, `prod/mc/` | Small shared leaf crates pulled in by the closure |
| `third_party/` | Vendored upstream source (Mermaid diagram stack) — see below |

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** — treat it as read-only. Prefer editing per-crate
> `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # always target specific crates; full-workspace builds are slow
cargo test -p xai-grok-config # per-crate tests
cargo clippy -p <crate>       # lint config: clippy.toml at the repo root
cargo fmt --all               # rustfmt.toml at the repo root
```

## Contributing

> [!NOTE]
> External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code in this repository is licensed under the **Apache License,
Version 2.0** — see [`LICENSE`](LICENSE).

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — crates.io / git dependencies,
  bundled UI themes, and **in-tree source ports** (including openai/codex and
  sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the codex and opencode ports (license texts +
  Apache §4(b) change notice)
- [`third_party/NOTICE`](third_party/NOTICE) — vendored Mermaid-stack index

<!-- UPSTREAM-CARRIED:END -->
