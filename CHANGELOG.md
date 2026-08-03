# Changelog

这是 `juya-ai-lab/grok-build` fork 的变更记录，面向使用者说明本 fork 的修复、分发和维护性变更。

- 上游运行时功能的原始说明仍以 `crates/codegen/xai-grok-shell/CHANGELOG.md` 和上游发布说明为准。
- 上游同步条目固定记录：上游版本/tag、上游 commit、`SOURCE_REV`、本 fork 同步 commit，以及隐私裁剪或冲突处理结论。
- `UPSTREAM_TRACE.md` 保存更详细的同步事实和维护决策；本文件只保留使用者需要的摘要。

## [Unreleased] — 2026-08-04

### Upstream 0.2.119 incremental sync

- 已从上游候选 `0.2.119`（`xai-org/grok-build@e5478eff`，`SOURCE_REV=27d2088ae3b3f25e9ddab462caa18a07005ada9a`）分批同步后台任务输出、nested checkout watcher、同分支 git-head 去重和子代理 watcher 覆盖判断；本 fork 对应主线提交为 `571c2d64`、`2d4eb18c`、`6c0f40d7`、`0d72ccd7`。
- 隐私审查批次 `2d9fbbde` 直接删除子代理配置/目录/凭据诊断日志、GCS 元数据和 prompt/permission/turn trace 上传构造，并加固 read-file、AGENTS tracker、LSP 与 workspace classifier 的 vendor-state 边界；本地 resume、推理、worktree 生命周期仍保留。
- 保留 fork 的 web-search 兼容修复：流式 `action`/`query` 可缺省，并支持 DeepSeek 复数 `queries`；`async-openai` 继续 pin 到 fork revision `7defed8a`。
- 版本号、`SOURCE_REV`、release/tag 和最终产物验证仍待完整上游 diff 审计完成后单独处理。

## [v0.2.118-fix1] — 2026-08-04

> 本 fork 修复 tag。当前源码已验证，但实际发行二进制和 npm/pnpm 产物仍待 smoke test；详见 [Issue #3](https://github.com/juya-ai-lab/grok-build/issues/3)。

### Bug fixes

- 修复多个 `[model.*]` 区块声明相同实际 `model` 值时，sibling slug propagation 覆盖显式 `api_backend` 的问题。
  - 功能修复：[040e3044](https://github.com/juya-ai-lab/grok-build/commit/040e3044d4e06da6bdccabc9e62e05b0dc2b9f1e)。
  - 同时覆盖 model entry 和 model provider 显式 backend；未显式配置 backend 的兼容继承行为保持不变。

### Distribution and maintenance

- README 增加 GitHub Releases 与 npm/pnpm 预编译安装说明，两种方式均为可选渠道。
- README 开头声明本 fork 特有文档，并用 `JUYA FORK MAINTAINED` / `UPSTREAM-CARRIED` 标记分隔 fork 说明与上游带来的内容。
- 建立 GitHub Issue 模板、来源/冲突/产物验证标签规则，并将 Issue #3 作为该问题的单一记录源。
- 删除本地 `juya-issue/` 档案，避免本地记录与 GitHub Issue 漂移。

### Upstream context

- 本版本仍基于上游 `v0.2.118`，没有把本 fork 文档治理变更冒充成新的上游同步。
- 上游基线：[xai-org/grok-build@780d1388](https://github.com/xai-org/grok-build/commit/780d1388fff103ff0db0d8c14de65af6225b4860)，`SOURCE_REV=64c4de99cc822b25ce9c54ab5a4f372093d0885d`。

## [v0.2.118] — 2026-08-03

### Upstream synchronization

- 同步上游 `v0.2.118`：上游同步 commit 为 [780d1388](https://github.com/xai-org/grok-build/commit/780d1388fff103ff0db0d8c14de65af6225b4860)，对应 `SOURCE_REV=64c4de99cc822b25ce9c54ab5a4f372093d0885d`。
- 本 fork 的同步与隐私裁剪提交为 [6a48278d](https://github.com/juya-ai-lab/grok-build/commit/6a48278d6ee4a4561e4bfce38de1a0ee2deb3914)；按仓库规则重新应用隐私边界裁剪并保留 fork 自持 CI。
- 随后修复同步后的确定性编译错误（[3cc9a6cd](https://github.com/juya-ai-lab/grok-build/commit/3cc9a6cda7fa616fb92acc2b453d1cdc9e18abcd)），并清理上游已移除且 fork 无调用者的旧兼容代码（[31920111](https://github.com/juya-ai-lab/grok-build/commit/3192011150357e462ba545465013197bc08f8ffe)）。

### Fork boundary

- 保持 Claude/Codex/Cursor 兼容面、工作区/会话制品上传、relay、提示词和工具详情外发、聚合遥测、错误上报及反馈路径的编译级禁用。
- 保留 `.agents/skills`、`.agents/commands`、OAuth 登录、正常推理、TUI 和会话管理等可用性范围。
