---
name: upstream-trace
description: Update the upstream trace of this privacy-trimmed fork — regenerate the git-derived blocks with scripts/upstream-trace.sh, then record the judgement content (merge/trim decisions, rationale) in UPSTREAM_TRACE.md. Use when syncing with upstream xai-org/grok-build, after local changes, or when asked about upstream status / whether upstream has updates / the trace.
---

# Upstream Trace

本仓库的"上游同步 + 本 fork 改动 + 临时事项"集中记录在 `UPSTREAM_TRACE.md`。
机械事实由脚本生成，判断性内容人工维护。流程如下：

## Steps

1. **刷新生成区块**：main 分支由 CI（`CI - Upstream Trace`）在每次 push 后自动运行 `scripts/upstream-trace.sh --fetch` 并回提交，无需手工执行；本地运行脚本仅用于预览或分支预演。生成区块（`<!-- TRACE:...:BEGIN/END -->` 标记之间）勿手改。

2. **读「当前状态」判定**：
   - `上游 SOURCE_REV 与本地一致` → 无新基线；树差异应全部为本 fork 本地裁剪。若出现无法归因于裁剪的差异路径，按内容核对。
   - `上游有新基线` → 上游有更新：按 AGENTS.md「上游合并规则」逐个评估——不触碰隐私边界 → 正常合入；触碰 → 裁剪移植（对照历史 port 提交，如 `6d0bfa9` / `bff0968` / `422ee9b`）。`.github/` 一律不采用上游 workflow。

3. **写「决策记录」**：在 `UPSTREAM_TRACE.md`「决策记录」追加一行：日期、上游 SHA（或本地 commit）、处理（合入/裁剪/忽略）、原因。上游同步与本地改动分开记。CI 会校验每个 fork 提交都有对应记录（本地可 `scripts/upstream-trace.sh --check` 预检；`chore: ... trace` 机械类提交豁免）。

4. **同步临时事项/注意事项**：如 async-openai 临时镜像移除、token 到期、npm hotfix 等有变化，更新对应小节（保持单点：内容指针指向 AGENTS.md / `.github/token-expiry.env`，不复制全文）。

5. **提交**：`chore: update upstream trace`（英文、单主题）；推送前与用户确认目标仓库与分支。

## 何时使用

- 上游 sync 后（无论合入还是裁剪）
- 本地功能改动后
- 用户问"上游有没有更新"时：先跑脚本再据实回答
