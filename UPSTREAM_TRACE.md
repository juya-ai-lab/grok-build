# 上游跟踪记录（Upstream Trace）

> 本文件是"上游同步 / 本 fork 改动 / 临时事项"的集中记录。
> **生成式部分**：`当前状态`、`上游同步日志`、`本 fork 改动日志` 三个区块（`<!-- TRACE:...:BEGIN/END -->` 标记之间）
> 由 `scripts/upstream-trace.sh` 用 git 事实整体重写，勿手改。push 到 main 后由 CI（`CI - Upstream Trace`）自动刷新并回提交，同时校验「决策记录」覆盖完整性（`scripts/upstream-trace.sh --check`）；本地也可手动运行预览/预检。
> **人工部分**：其余小节（宗旨、临时事项、注意事项、决策记录）为判断性内容，脚本与 CI 都不会触碰。

## 宗旨摘要

本仓库是 [xai-org/grok-build](https://github.com/xai-org/grok-build) 的隐私裁剪 fork，完整原则见 [AGENTS.md](AGENTS.md)。核心三条：

1. **隐私边界优先（编译级）**：硬禁用 Claude/Codex/Cursor 兼容、制品上传、relay 同步、external OTEL 提示词/工具字段、聚合遥测、错误上报、反馈路径——必须是编译级 kill switch，不是运行时开关。
2. **可用性不缩水**：`.agents/skills`、`.agents/commands`、OAuth 登录与正常推理全部保留。
3. **自持可跟进**：版本单点 = `crates/codegen/xai-grok-version/Cargo.toml`；基线 = `SOURCE_REV`。

## 当前状态

<!-- TRACE:status:BEGIN -->
- 本地 HEAD: 6c0f40d7 (2026-08-04)
- 版本: 0.2.118
- 上游基线 SOURCE_REV: 64c4de99cc822b25ce9c54ab5a4f372093d0885d
- 上游 upstream/main: e5478eff (2026-08-03, Synced from monorepo)
- 上游 SOURCE_REV: 27d2088ae3b3f25e9ddab462caa18a07005ada9a
- 落后上游 1 提交 / 本 fork 领先 100 提交; 树差异 220 个文件
- 判定: 上游有新基线：SOURCE_REV 变为 27d2088ae3b3f25e9ddab462caa18a07005ada9a，需按 AGENTS.md 评估合入/裁剪
<!-- TRACE:status:END -->

## 上游同步日志

每次上游 sync 一行（由脚本生成；`SOURCE_REV` 为该提交对应的 monorepo 基线；"处理"结论见下方「决策记录」）。

<!-- TRACE:sync:BEGIN -->
| 上游提交 | 日期 | 内容 | SOURCE_REV |
|---|---|---|---|
| e5478eff | 2026-08-03 | Synced from monorepo | 27d2088ae3b3f25e9ddab462caa18a07005ada9a |
| 780d1388 | 2026-08-03 | Synced from monorepo | 64c4de99cc822b25ce9c54ab5a4f372093d0885d |
| a4221165 | 2026-07-31 | Synced from monorepo | 8d69c91f02bcacf01e98d5aebbf2f92547c45738 |
| dd04f397 | 2026-07-30 | Synced from monorepo | 2a28b4a86cfc4a4c133c35b7fc2a6a9964387c39 |
| 500129c7 | 2026-07-29 | Synced from monorepo | 6372e41d828b8a6ee82c29e01a69e27ec895cca9 |
| 5da6962e | 2026-07-28 | Synced from monorepo | 2a818575225183d8ca915f5632a09b8067b5156a |
| 02d93594 | 2026-07-27 | Synced from monorepo | 1adcd1f477870e4a97bacbd6be78c8a3bfbac46d |
| b41c75a5 | 2026-07-26 | Synced from monorepo | 91d8cf309110a3b879c1b8198f7525aed545dfb4 |
| 47348d13 | 2026-07-25 | Synced from monorepo | d02693a856a54f1030695b36b91d276e96b30b23 |
| 6e386420 | 2026-07-24 | Synced from monorepo | 9b8d35b46d959c042ea9aa31cbbebbd1f0c5c527 |
| 69f0ba88 | 2026-07-23 | Synced from monorepo | 95d84f443eddcbed6cbfd6eed22e2eafe6b3939d |
| a5727c59 | 2026-07-22 | Synced from monorepo | 30192d2eef5d91a8fff0e53957de5bd05b43398c |
| 3af4d5d3 | 2026-07-21 | Synced from monorepo | 0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899 |
| a881e670 | 2026-07-20 | Synced from monorepo | c5c4ce03436b4bb2cec43d3feaa27dee0109bf37 |
| ba76b0a6 | 2026-07-19 | Synced from monorepo | ba69d70c2f7d70a130a323b2becdf137af784c7f |
| 7cfcb20d | 2026-07-18 | Synced from monorepo | f9736c7b86f8e1c0e99e20ebbbd1195cd0c147e3 |
| 98c3b243 | 2026-07-17 | Synced from monorepo | 124d85bc5dc6e7805560215fcc6d5413944920e1 |
| 8adf9013 | 2026-07-16 | Synced from monorepo | 2ec0f0c8488842da03a71eeee3c61154957ca919 |
| c68e39f6 | 2026-07-16 | Publish harness and TUI open-source | n/a |
<!-- TRACE:sync:END -->

## 本 fork 改动日志

本 fork 相对上游的全部提交（由脚本生成；原因/对应宗旨见「决策记录」）。

<!-- TRACE:fork:BEGIN -->
| 提交 | 日期 | 内容 |
|---|---|---|
| 6c0f40d7 | 2026-08-04 | fix: refresh git head after same-branch commits |
| 2d4eb18c | 2026-08-04 | fix: avoid watching nested workspaces |
| 15e800b8 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| ddbee923 | 2026-08-04 | docs: record incremental upstream batch |
| 571c2d64 | 2026-08-04 | fix: show full size for partial task output |
| 9ee5aeec | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 1b38e1d3 | 2026-08-04 | docs: record pending upstream privacy audit |
| 6bc2ec3c | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 4e5100d5 | 2026-08-04 | docs: require full privacy audit for upstream sync |
| 427e4490 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| b6d5a672 | 2026-08-04 | docs: mark fork and upstream README sections |
| 8c9f55da | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 85c12155 | 2026-08-04 | docs: add fork changelog with upstream provenance |
| 2d5810cc | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 00e900c7 | 2026-08-04 | docs: document evidence-based issue labels |
| cb53eced | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 0b2c7c89 | 2026-08-04 | docs: keep npm guidance outside artifact choices |
| 584b2265 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| c06d7525 | 2026-08-04 | docs: separate npm package artifact choice |
| 45ecce9f | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 037b8140 | 2026-08-04 | docs: default npm package in artifact checklist |
| aaf80936 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| a0fd99af | 2026-08-04 | docs: list selectable artifact variants in issue template |
| f045babb | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 8a5805cc | 2026-08-04 | docs: simplify artifact choices in issue template |
| df0dac6b | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 4939bf35 | 2026-08-04 | docs: make artifact selection conditional |
| e1ac7215 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| d399c4dc | 2026-08-04 | docs: add artifact choices to issue template |
| 89e3c151 | 2026-08-04 | docs: align issue and release records with repository conventions |
| 373064d4 | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 8d53f18d | 2026-08-04 | docs: migrate issue tracking to GitHub and add issue template |
| babe3c4e | 2026-08-04 | chore: refresh upstream trace [skip ci] |
| 040e3044 | 2026-08-04 | fix: preserve per-entry api_backend for duplicate model slugs |
| 4267b67a | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 31920111 | 2026-08-03 | chore: remove obsolete Claude change detector |
| 9688db9a | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 1c1f6956 | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 3cc9a6cd | 2026-08-03 | fix: restore debug tracing import for Claude import state |
| f45b3fd4 | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 4034e5a6 | 2026-08-03 | docs: record upstream merge decisions and collaboration context |
| 6a48278d | 2026-08-03 | Merge upstream 0.2.118 sync with privacy trims |
| a49a236d | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 59f2c19d | 2026-08-03 | chore: update upstream trace |
| a26df1c1 | 2026-08-03 | fix: make trace coverage check robust to short-sha length |
| 0af2361f | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 07cd629b | 2026-08-03 | chore: update upstream trace |
| 1adbe5e6 | 2026-08-03 | chore: refresh upstream trace [skip ci] |
| 5aa77501 | 2026-08-03 | chore: add script-driven upstream trace with CI auto-refresh and coverage check |
| bfa46df8 | 2026-08-03 | docs: add npm/pnpm global install note to README |
| 9d9e509f | 2026-08-02 | ci(npm): retry E409 registry race and tolerate tombstoned versions |
| d62623a1 | 2026-08-01 | fix: support DeepSeek plural web search queries |
| 777fbf46 | 2026-08-01 | fix: tolerate missing query on streamed web search calls |
| d9fc97cd | 2026-08-01 | fix: tolerate missing action on streamed web search calls |
| 767f3a6a | 2026-08-01 | ci: track token expiry dates and remind before rotation |
| f9f64558 | 2026-08-01 | fix(npm): decouple release asset version from npm package version |
| ede7b088 | 2026-08-01 | ci(npm): support version and dist-tag overrides for hotfix publishes |
| b062b230 | 2026-08-01 | fix(npm): expose grok as the CLI command |
| 903a6c66 | 2026-08-01 | ci: publish prebuilt binaries to npm after release |
| 1b611506 | 2026-08-01 | ci: satisfy shellcheck in release notes generation |
| dac1977c | 2026-08-01 | ci: fix macOS checksum and dedupe release notes |
| 21753026 | 2026-08-01 | ci: use dedicated token for release creation |
| 64b5d144 | 2026-08-01 | ci: satisfy shellcheck in sccache env setup |
| 4377dccf | 2026-08-01 | ci: cap sccache cache size to fit repo cache budget |
| 98d31e80 | 2026-08-01 | ci: publish release assets per platform as they finish |
| 0838904b | 2026-08-01 | ci: resolve format check toolchain from rust-toolchain.toml |
| 0172b44d | 2026-08-01 | Merge branch port/upstream-0.2.117 into main |
| 15c8c61c | 2026-08-01 | ci: let format check follow the repository toolchain |
| f8ebdfbe | 2026-08-01 | port: merge upstream 0.2.117 sync with privacy trims |
| 401a0864 | 2026-08-01 | docs: add AGENTS.md capturing fork principles and merge rules |
| 353933dc | 2026-08-01 | ci: skip sccache on Windows and upgrade actions/cache to node24 |
| 35102ee7 | 2026-08-01 | ci: separate manual dispatch runs in concurrency groups |
| 60e0d84d | 2026-08-01 | ci: pin zizmor/cargo-deny versions and scope release concurrency |
| 5a88ec74 | 2026-08-01 | ci: cache tool downloads and rust toolchain in lint jobs |
| 6eb2edd7 | 2026-08-01 | ci: harden workflows and add dependency audit checks |
| 634a3dcc | 2026-08-01 | ci: keep the check workflow to formatting only |
| 0bcc2317 | 2026-08-01 | ci: add lightweight fmt/check workflow |
| 7c1d67e6 | 2026-08-01 | ci: cap sccache size per release target |
| de0a82a5 | 2026-08-01 | ci: fail manual releases when the tag points elsewhere |
| e844fbab | 2026-08-01 | ci: print sccache stats after release builds |
| a402686e | 2026-08-01 | ci: cache protoc installs across release runs |
| 76cb0a34 | 2026-08-01 | ci: cache release compiles with sccache |
| 983a855a | 2026-08-01 | ci: create release tag up front and share resolved version |
| 1e6f0824 | 2026-07-31 | ci: make version crate manifest path configurable |
| 80c0b815 | 2026-07-31 | ci: derive manual release tag from the crate version |
| 18ed28f6 | 2026-07-31 | Merge branch 'ci/tag-release' into main |
| 987822ff | 2026-07-31 | Merge branch 'port-upstream' into main |
| 3c17dd0a | 2026-07-31 | ci: auto-resolve latest tag for manual release runs |
| d286045e | 2026-07-31 | ci: allow manual release runs via a tag input |
| 2074bb9d | 2026-07-31 | ci: support manual release re-run via workflow_dispatch |
| 38f87fae | 2026-07-31 | style: run cargo fmt --all on the merged port |
| f36387fa | 2026-07-31 | port: add privacy-fork CI files |
| 8afeb2c7 | 2026-07-31 | port: pager, docs, README, upstream surface audit |
| 422ee9ba | 2026-07-31 | port: remove Claude/Codex/Cursor compatibility surfaces |
| bff09687 | 2026-07-31 | port: gate telemetry, uploads, feedback, error reporting and their consumers |
| 6d0bfa9d | 2026-07-31 | port: compile-time kill switches, path validators, relay gates, deps |
| 2eed7e74 | 2026-07-20 | ci: add multi-platform GitHub Actions release workflow |
| a5c85bad | 2026-07-20 | style: apply rustfmt to privacy-gate follow-up edits |
| 4a6004e2 | 2026-07-19 | Harden privacy boundaries and preserve shared agent commands |
| 7190b3f7 | 2026-07-19 | Disable vendor compatibility and content uploads |
<!-- TRACE:fork:END -->

## 临时事项与到期项

| 事项 | 内容 | 触发条件/到期 | 指针 |
|---|---|---|---|
| async-openai 临时镜像 | 继续 pin `juya-ai-lab/async-openai@7defed8`（镜像 xAI fork 基座 `95b52eb` + `action`/`query` 可选及 DeepSeek 复数 `queries` backport） | 上游同时具备三项兼容能力后对齐并移除；当前 `our-forks@884aff` 只解决 `action` | AGENTS.md「async-openai 依赖」 |
| CI token 到期 | release / dist 用 token 的到期日集中登记 | release/dist workflow 启动时检查：≤30 天 warning，过期 fail；换 token 时同步更新 | `.github/token-expiry.env` |
| 上游 0.2.119 待全量隐私审计 | `upstream/main=e5478eff`，`SOURCE_REV=27d2088ae3b3f25e9ddab462caa18a07005ada9a`；相对 `780d1388` 涉及 72 个文件，包含新增技能触发遥测和 watcher 改动 | 完成整个 diff、调用链、依赖及发布路径审计，重新应用隐私裁剪并验证后，才能决定是否合入 | 上游同步候选；当前未合入 |

## 注意事项

- 上游历史会被 monorepo sync 重写，commit 无法直接追溯 → 上游对比一律按内容（文件 diff），不依赖 commit SHA 可达性。
- `.github/` 一律用本仓库 CI，不采用上游 workflow。
- Windows 编译不走 sccache（其 server 在巨型 crate 上会崩）。
- npm 包版本镜像 release tag（如 `0.2.117` ↔ `v0.2.117`）；打包 hotfix 用 `-fix.N` 后缀。
- 拿不准的隐私边界改动：停下来问，不要替用户做决定。

## 决策记录

人工维护：每次上游同步或本地改动，在此追加一行"原因/裁剪说明"，用 commit 或上游 SHA 作锚点。

### 本地改动记录（覆盖 fork 全部历史，按主题分组）

| 提交（SHA） | 内容 | 原因/对应宗旨 |
|---|---|---|
| `7190b3f7` `4a6004e2` `a5c85bad` `6d0bfa9d` `bff09687` `422ee9ba` `8afeb2c7` `f36387fa` `38f87fae` | 隐私裁剪 port 系列：禁用 vendor 兼容与内容上传、编译级 kill switch/路径校验/relay 门控、遥测/上传/反馈/错误上报门控、移除 Claude/Codex/Cursor 兼容面、pager/文档审计、fork CI 文件、fmt 收尾 | 宗旨 1（隐私边界编译级，不可协商）+ 2（可用性不缩水）；即 AGENTS.md 引用的 port 提交 `6d0bfa9`/`bff0968`/`422ee9b` |
| `2eed7e74` `2074bb9d` `d286045e` `3c17dd0a` `80c0b815` `1e6f0824` `983a855a` `de0a82a5` | release workflow 基建：多平台发布、手动触发（tag 输入/自动解析）、版本 crate 路径可配、先建 tag 共享解析版本、防 tag 指向错误 | AGENTS.md「Release」：网页手动发版；版本单点可配 |
| `76cb0a34` `a402686e` `e844fbab` `7c1d67e6` `353933dc` `4377dccf` `64b5d144` | 构建缓存：sccache/protoc 缓存、sccache 统计、按目标限容、Windows 跳过 sccache、缓存预算限容、shellcheck 修复 | 资源敏感：缓存要有效但不能无限膨胀 |
| `0bcc2317` `634a3dcc` `6eb2edd7` `5a88ec74` `60e0d84d` `35102ee7` `15c8c61c` `0838904b` | format/lint/依赖审计：仅 fmt 的轻量检查、workflow 加固与 cargo-deny、工具/toolchain 缓存、zizmor/cargo-deny 版本锁定、并发分组、toolchain 跟随 `rust-toolchain.toml` | AGENTS.md「CI 约定」：独立、不阻塞发布 |
| `987822ff` `18ed28f6` `0172b44d` | 分支合并（port-upstream / ci/tag-release / port/upstream-0.2.117） | 合入记录 |
| `f8ebdfbe` | 合入上游 0.2.117 sync 并按 AGENTS.md 重新应用隐私裁剪 | 「上游合并规则」：上游改动 → 逐个重新裁剪 |
| `401a0864` | AGENTS.md：fork 原则、合并规则、CI 约定文档化 | 宗旨 3（自持可跟进）：规则单点、决策留痕 |
| `98d31e80` | 各平台 release 产物完成后即发布 | 发布速度与可观察性 |
| `21753026` | release 用专用 token | CI 安全：凭证最小化 |
| `dac1977c` `1b611506` | release notes 生成修复（macOS 校验和、shellcheck） | 发布流程可靠、可复现 |
| `767f3a6a` | token 到期日期登记与提醒 | CI 可维护：凭证轮换不靠记忆 |
| `903a6c66` `b062b230` `ede7b088` `f9f64558` `9d9e509f` | npm 分发：发布预编译二进制、暴露 `grok` 命令、版本/dist-tag 覆盖、资产/包版本解耦、E409 重试 | 分发渠道：npm/pnpm 全局安装 |
| `6a48278d` | 合入 upstream `0.2.118` / `SOURCE_REV=64c4de99`；冲突文件按 AGENTS.md 重新应用隐私裁剪，保留 fork CI，并对新增资源遥测发送路径加聚合遥测编译门控 | 上游合并规则 + 宗旨 1（隐私边界编译级）；全量 cargo check 因资源风险停止，已完成 fmt 与轻量 metadata 验证 |
| `3cc9a6cd` | 修复上游合并冲突遗漏的 `tracing::debug` 导入；CI Release 在 6 个平台均于 `xai-grok-shell` 编译阶段报同一错误，补回导入后将 `v0.2.118` 移到此代码提交 | 修复 release 阻断的确定性编译错误；不改变隐私边界或功能行为 |
| `31920111` | 删除上游 `780d1388` 已移除且 fork 当前无调用者的 `has_new_changes`；保留 `CLAUDE_CODE_COMPAT_ENABLED=false` 的隐私门控 | 对齐上游并清理无效兼容代码；不改变默认功能或隐私边界 |
| `4034e5a6` | 记录跨模型协作时以当前指令、仓库事实和决策文档为准；补充本次 async-openai 上游复核结论 | 宗旨 3（协作上下文与判断性决策可追溯），不重新引入模型身份切换描述 |
| `d9fc97cd` `777fbf46` `d62623a1` | 容忍流式 web_search 缺 `action`/`query`、支持 DeepSeek 复数 `queries` | async-openai 临时镜像兼容层（见「临时事项」） |
| `780d1388` / `our-forks@884aff` / `juya@e03c366` | 上游同步后的 async-openai 复核：`our-forks` 已吸收 `action` 可选，但仍缺 `query` 可选和 DeepSeek `queries`；`juya@e03c366` 只是合并已有 `7defed8` 内容 | 保留 `juya-ai-lab/async-openai@7defed8` 内容 pin，避免恢复后重新出现 `missing field query` 与 DeepSeek `missing field queries`；待剩余两项上游化后再移除 |
| `bfa46df8` | README 增加 npm/pnpm 全局安装说明 | 文档：配合 npm 分发 |
| `5aa77501` | 建立脚本驱动的上游 trace 机制：`scripts/upstream-trace.sh` 生成机械事实 + CI 自动刷新回提交 + 决策记录覆盖校验（`--check`）+ `.grok/skills/upstream-trace` 固化流程 + AGENTS.md 规则 | 宗旨 3：决策留痕、机械事实不靠人记；CI 接管刷新与校验 |
| `a26df1c1` | 修复覆盖校验对短 SHA 长度的依赖：改按完整 SHA 前缀匹配（`%h` 长度随仓库对象数变化，CI 与本地不一致会误报）；生成区统一 8 位显示 | 机制健壮性：跨环境可复现 |
| `040e3044` | 修复同一实际 model slug 下显式 `api_backend` 被 sibling propagation 覆盖的问题，并覆盖 `[model.*]` 与 `[model_providers.*]` 两种配置来源 | JUYA-ISSUE-001；保持显式 provider 路由独立，同时保留未显式配置 backend 的兼容继承行为 |
| `8d53f18d` | 将 issue #3 的原始描述、调查、修复和测试证据统一迁移到 GitHub，删除本地 issue 档案，并补充 upstream/fork provenance 模板与预编译安装说明 | 宗旨 3：可追溯协作与单一记录源；不改变隐私边界或运行时行为 |
| `89e3c151` | 按 `.grok/skills/upstream-trace`、`AGENTS.md` 和 release/npm workflow 收敛 issue 指引、trace 锚点与 npm 版本说明；不改变运行时行为或上游代码 | 宗旨 3：记录可追溯且不把 fork 分发事实误写成上游事实 |
| `d399c4dc` | 将当前 release matrix 的六个平台二进制、npm/pnpm JS wrapper、源码构建和其他产物纳入 Issue 勾选项；npm/pnpm 明确为同一产品的安装方式 | 宗旨 3：让产物级问题可定位；列表依据本 fork workflow，不扩展上游代码 |
| `4939bf35` | 允许源码、配置、文档、CI 或上游同步等非产物问题明确选择“不涉及产物”，并为尚未判断的报告保留独立状态 | 宗旨 3：记录真实范围，不把所有 issue 强行归类为产物问题 |
| `8a5805cc` | 将产物勾选收敛为“直接平台产物”和 npm package 两个用户可见类别；保留平台/架构文本字段，不再细分 npm 与 pnpm | 宗旨 3：分类足够定位问题，同时避免模板过度细化 |
| `a0fd99af` | 将六个 release 平台二进制恢复为直接可勾选项，npm/pnpm 仍合并为一个 npm package 选项；特殊产物继续通过 Other 和正文补充 | 宗旨 3：让常见产物可直接归类，同时保留非产物和特殊情况入口 |
| `037b8140` | 考虑 npm wrapper 通常分发或调用同一平台产物，将 npm package 设为默认勾选；模板同时要求在明确不涉及时取消，避免把默认推断当成事实 | 宗旨 3：按实际影响链提高默认覆盖率，同时保留准确修正入口 |
| `c06d7525` | 将 npm package 独立为单独小节并取消默认勾选；只有报告者确认受影响时才选择，避免模板预设替代调查结论 | 宗旨 3：产物范围记录保持准确、可复核 |
| `0b2c7c89` | 按最终分类约定移除 npm package 产物 checkbox，改为独立说明和正文补充项；直接平台产物列表保持可勾选 | 宗旨 3：产物分类保持简洁，npm 使用细节仍可追溯 |
| `00e900c7` | 增加 `upstream-originated`、`upstream-conflict`、`fork-specific` 和 `needs-artifact-validation` 标签定义；模板仅默认通用 `bug`，scope/status 标签由维护者按证据添加 | 宗旨 3：让问题来源、fork/上游冲突和产物验证状态可筛选且不被模板预设误导 |
| `85c12155` | 新增根目录 fork-level `CHANGELOG.md`，面向使用者记录修复/分发变更，并要求上游条目同时写明版本、commit、`SOURCE_REV` 和本 fork 同步 commit；详细证据仍集中在 `UPSTREAM_TRACE.md` | 宗旨 3：使用者可读的变更摘要与维护者追踪记录分层，避免把 fork 变更误写成上游变更 |
| `b6d5a672` | README 开头声明本 fork 特有的根目录文档，并用 `JUYA FORK MAINTAINED` / `UPSTREAM-CARRIED` 标记分隔 fork 说明与上游带来的内容 | 宗旨 3：同步上游 README 时能识别维护边界，避免覆盖隐私、分发和协作说明 |
| `4e5100d5` | 将上游同步的隐私审计范围提升为整个 diff、调用链、依赖、构建与发布路径，不以历史裁剪清单替代全量审计；未决项需记录并暂停合入 | 宗旨 1（隐私边界编译级）+ 宗旨 3（同步决策可追溯） |
| `1b38e1d3` | 记录上游 `0.2.119` 候选的 `e5478eff` / `SOURCE_REV=27d2088a`，明确其尚未完成全量隐私审计，暂不视为已同步 | 宗旨 1（隐私边界编译级）+ 宗旨 3（同步状态可追溯） |

| 571c2d64 | fix: show full size for partial task output；从上游 0.2.119 候选选择性移植后台任务部分日志的真实总大小提示，仅改 xai-grok-tools 输出格式和测试；不更新版本号或 SOURCE_REV | B1 低风险独立批次：无新增外部通信、遥测、持久化、路径扫描或凭证读取；模型额外看到完整输出字节数，已通过 fmt、针对性测试和 xai-grok-tools 全量 lib 回归；其余 0.2.119 改动仍待全量隐私审计 |
| `2d4eb18c` | 手工合入 upstream 0.2.119 的 nested checkout watcher 逻辑：识别未声明的 clone/worktree/Sapling checkout，保留 submodule，避免 watcher 覆盖其它 workspace；同时保留 fork 的 vendor hard-deny、symlink 防护和 `.agents/.grok` 可用边界，并修正当前 watcher 根含 `.git` 时的误剪枝风险 | 上游批次 B2-W；宗旨 1（路径/隐私边界优先）+ 宗旨 2（本地 workspace watcher 功能保持）；新增逻辑仅做本地 metadata/git 判断，无上传/遥测/凭证路径；`xai-fsnotify` 定向测试 129 passed、15 ignored，尚未更新版本或 SOURCE_REV |
| `6c0f40d7` | 合入上游同分支 commit 后刷新已有 `x.ai/git_head_changed` 的本地 dedup key；将当前 commit 纳入身份，避免同一 branch 上新 commit 被错误去重；不改变通知 payload 或 opt-in | 上游批次 B2-GH；宗旨 2（本地会话/状态刷新功能）+ 宗旨 1（无新增上传/遥测）；`git_head_dedup_key_identity` 1 passed、slug_propagation 6 passed，尚未更新版本或 SOURCE_REV |
| ddbee923 | docs: record incremental upstream batch；刷新生成式 trace 区块并记录 B1 的独立合入、测试和隐私结论 | 宗旨 3：上游分批决策和验证证据可追溯；不改变运行时行为 |

### 上游同步记录（未来在此追加）

| 日期 | 上游 SHA | 处理（合入/裁剪/忽略） | 说明 |
|---|---|---|---|
| （待上游有更新时记录） | | | |
