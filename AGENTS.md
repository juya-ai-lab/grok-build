# AGENTS.md

本仓库是 [xai-org/grok-build](https://github.com/xai-org/grok-build) 的隐私裁剪 fork（juya-ai-lab/grok-build）。我们跟随上游功能，但固定关闭一切会把工作区内容外送的通道。任何改动（包括合并上游）都受以下原则约束。

## 不可违背原则

### 1. 隐私边界优先（编译级，不可协商）

硬禁用，且必须是编译级 kill switch / 源码删除，不能是运行时开关或配置项：

- Claude / Codex / Cursor 兼容表面（导入、别名、权限、设置读取等）
- 自动会话/工作区制品上传（gcs/trace 等 upload 通道）
- relay 同步
- external OTEL 中的提示词和工具详情字段
- 聚合元数据遥测、错误上报、反馈路径（feedback_manager、sentry 等）

### 2. 可用性不缩水

只裁"送数据出去"的部分，不裁功能。必须保留：

- `.agents/skills` 与 `.agents/commands`
- OAuth 登录与正常推理
- TUI / pager / 会话管理等原版体验

### 3. 自持且可跟进

- 版本号单点事实：`crates/codegen/xai-grok-version/Cargo.toml`
- release 流程的 `VERSION_CRATE_MANIFEST` 指向它（放在 workflow 文件靠前，路径可变）
- 上游基线记录在 `SOURCE_REV`

## 上游合并规则

- 基线 = `SOURCE_REV` 记录的提交。注意：上游历史会被 monorepo sync 重写，commit 可能无法直接追溯，此时按内容（文件 diff）对比。
- 上游新功能/修复：不触碰隐私边界 → 正常合入；触碰 → 裁剪移植（对照历史 port 提交，如 `6d0bfa9` / `bff0968` / `422ee9b`）。
- 上游改动了本地裁剪过的文件 → 逐个重新应用裁剪。
- 上游新增文件 → 先检查是否敏感路径（上传/导出/遥测/兼容），敏感则裁。
- `Cargo.toml` / `Cargo.lock`：合入后 `cargo check` + `cargo deny check` 验证；`deny.toml` 许可证白名单按需补充。
- `.github/` 一律保留本仓库的 CI，不采用上游 workflow。
- 版本：上游 bump 后 release tag 自动跟随（`vX.Y.Z`）；若版本 crate 路径变化，更新 `VERSION_CRATE_MANIFEST`。
- `async-openai` 依赖（临时改动）：pin 到 `juya-ai-lab/async-openai`（镜像 xAI fork 基座 `95b52eb` + #548 backport：`WebSearchToolCall.action` 与 `WebSearchActionSearch.query` 可选，并兼容 DeepSeek 复数 `action.search.queries`；修复流式 `web_search_call` 偶发 `missing field action` / `missing field query` 及 DeepSeek 400 `missing field queries`）。本次同步时 xai-org 仍 pin `our-forks` 的 `95b52eb`；`our-forks` 当前 `884aff` 虽已吸收 `action` 可选修复，但仍未使 `query` 可选，也未兼容 DeepSeek 复数 `queries`，所以本 pin 继续保留，待三项能力都上游化后再移除。

## CI 约定（本仓库自持）

- **Release**：网页手动发版（填 tag，或留空自动取版本 crate 版本号）+ push tag 触发；6 平台产物；sccache 仅 Linux/macOS（Windows 直接编译，其 sccache server 在巨型 crate 上会崩）；protoc/registry/sccache 有缓存；所有 action 锁 commit SHA。
- **Format Check**：仅 rustfmt，路径过滤 + concurrency。
- **Workflow and Dependency Checks**：actionlint + zizmor + cargo-deny，独立、不阻塞发布，路径过滤 + concurrency。
- **Upstream Trace**：push 到 main 后自动跑 `scripts/upstream-trace.sh --fetch`，刷新 `UPSTREAM_TRACE.md` 生成区块并回提交（commit 带 `[skip ci]` 防循环）；push/PR 上校验「决策记录」覆盖完整性（`--check`，机械刷新类提交豁免）；判断性内容（决策记录等）人工维护。
- 命名：workflow/job/step 用 `CI - xxx` 纯文字前缀、Title Case，无冒号、无 emoji。
- 工具版本锁定：zizmor / cargo-deny / actionlint 固定版本，升级手动并验证。
- Token 到期：到期日期集中在 `.github/token-expiry.env`，release/dist workflow 启动时检查（≤30 天 warning，过期 fail）；换 token 时同步更新该文件与仓库 secret。

## 工作习惯约定

- 仓库记录干净：测试不留 tag / 分支 / run 痕迹，测试后清理。
- Rust 代码改动后：先对受影响 crate 运行 cargo fmt --package <package> -- --check；发现格式差异先格式化，再运行定向测试；不要格式化无关文件。
- 上游同步或本地改动后，更新 `UPSTREAM_TRACE.md`：机械事实（当前状态/同步日志/改动日志）由 `scripts/upstream-trace.sh` 生成、push 到 main 后 CI 自动刷新；人工职责是判断性内容——「决策记录」（原因/裁剪说明）与临时事项（async-openai 镜像、token 到期等）。完整流程见 `.grok/skills/upstream-trace`。
- 外部写操作（推送、删 tag、发布 release）先确认目标仓库与分支。
- commit message 用英文、规范，一个提交一个主题。
- 资源敏感：轻量、增量检查优先；不做全量检查、prewarm、apt 缓存等重流程；缓存要有效但不能无限膨胀。
- 拿不准的隐私边界改动，停下来问，不要替用户做决定。

## 协作上下文

- 上下文与对话历史可能跨模型传递；后续协作以当前用户的明确指令、仓库当前状态和本文件/`UPSTREAM_TRACE.md` 的记录为准，不臆测前一模型未记录的意图。
- 如历史对话与仓库事实或当前指令冲突，先指出冲突；涉及隐私边界的未决事项暂停并向用户确认。
