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
- 本地 HEAD: bfa46df (2026-08-03)
- 版本: 0.2.117
- 上游基线 SOURCE_REV: 8d69c91f02bcacf01e98d5aebbf2f92547c45738
- 上游 upstream/main: a422116 (2026-07-31, Synced from monorepo)
- 上游 SOURCE_REV: 8d69c91f02bcacf01e98d5aebbf2f92547c45738
- 落后上游 17 提交 / 本 fork 领先 13 提交; 树差异 155 个文件
- 判定: 无新基线：上游 SOURCE_REV 与本地一致；树差异 155 个文件，应为本地裁剪，需人工核对
<!-- TRACE:status:END -->

## 上游同步日志

每次上游 sync 一行（由脚本生成；`SOURCE_REV` 为该提交对应的 monorepo 基线；"处理"结论见下方「决策记录」）。

<!-- TRACE:sync:BEGIN -->
| 上游提交 | 日期 | 内容 | SOURCE_REV |
|---|---|---|---|
| a422116 | 2026-07-31 | Synced from monorepo | 8d69c91f02bcacf01e98d5aebbf2f92547c45738 |
| dd04f39 | 2026-07-30 | Synced from monorepo | 2a28b4a86cfc4a4c133c35b7fc2a6a9964387c39 |
| 500129c | 2026-07-29 | Synced from monorepo | 6372e41d828b8a6ee82c29e01a69e27ec895cca9 |
| 5da6962 | 2026-07-28 | Synced from monorepo | 2a818575225183d8ca915f5632a09b8067b5156a |
| 02d9359 | 2026-07-27 | Synced from monorepo | 1adcd1f477870e4a97bacbd6be78c8a3bfbac46d |
| b41c75a | 2026-07-26 | Synced from monorepo | 91d8cf309110a3b879c1b8198f7525aed545dfb4 |
| 47348d1 | 2026-07-25 | Synced from monorepo | d02693a856a54f1030695b36b91d276e96b30b23 |
| 6e38642 | 2026-07-24 | Synced from monorepo | 9b8d35b46d959c042ea9aa31cbbebbd1f0c5c527 |
| 69f0ba8 | 2026-07-23 | Synced from monorepo | 95d84f443eddcbed6cbfd6eed22e2eafe6b3939d |
| a5727c5 | 2026-07-22 | Synced from monorepo | 30192d2eef5d91a8fff0e53957de5bd05b43398c |
| 3af4d5d | 2026-07-21 | Synced from monorepo | 0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899 |
| a881e67 | 2026-07-20 | Synced from monorepo | c5c4ce03436b4bb2cec43d3feaa27dee0109bf37 |
| ba76b0a | 2026-07-19 | Synced from monorepo | ba69d70c2f7d70a130a323b2becdf137af784c7f |
| 7cfcb20 | 2026-07-18 | Synced from monorepo | f9736c7b86f8e1c0e99e20ebbbd1195cd0c147e3 |
| 98c3b24 | 2026-07-17 | Synced from monorepo | 124d85bc5dc6e7805560215fcc6d5413944920e1 |
| 8adf901 | 2026-07-16 | Synced from monorepo | 2ec0f0c8488842da03a71eeee3c61154957ca919 |
| c68e39f | 2026-07-16 | Publish harness and TUI open-source | n/a |
<!-- TRACE:sync:END -->

## 本 fork 改动日志

本 fork 相对上游的全部提交（由脚本生成；原因/对应宗旨见「决策记录」）。

<!-- TRACE:fork:BEGIN -->
| 提交 | 日期 | 内容 |
|---|---|---|
| bfa46df | 2026-08-03 | docs: add npm/pnpm global install note to README |
| 9d9e509 | 2026-08-02 | ci(npm): retry E409 registry race and tolerate tombstoned versions |
| d62623a | 2026-08-01 | fix: support DeepSeek plural web search queries |
| 777fbf4 | 2026-08-01 | fix: tolerate missing query on streamed web search calls |
| d9fc97c | 2026-08-01 | fix: tolerate missing action on streamed web search calls |
| 767f3a6 | 2026-08-01 | ci: track token expiry dates and remind before rotation |
| f9f6455 | 2026-08-01 | fix(npm): decouple release asset version from npm package version |
| ede7b08 | 2026-08-01 | ci(npm): support version and dist-tag overrides for hotfix publishes |
| b062b23 | 2026-08-01 | fix(npm): expose grok as the CLI command |
| 903a6c6 | 2026-08-01 | ci: publish prebuilt binaries to npm after release |
| 1b61150 | 2026-08-01 | ci: satisfy shellcheck in release notes generation |
| dac1977 | 2026-08-01 | ci: fix macOS checksum and dedupe release notes |
| 2175302 | 2026-08-01 | ci: use dedicated token for release creation |
<!-- TRACE:fork:END -->

## 临时事项与到期项

| 事项 | 内容 | 触发条件/到期 | 指针 |
|---|---|---|---|
| async-openai 临时镜像 | pin 到 `juya-ai-lab/async-openai`（镜像 xAI fork 基座 `95b52eb` + #548 backport，兼容 DeepSeek 复数 `queries`） | 上游升级/修复后对齐并移除 | AGENTS.md「async-openai 依赖」 |
| CI token 到期 | release / dist 用 token 的到期日集中登记 | release/dist workflow 启动时检查：≤30 天 warning，过期 fail；换 token 时同步更新 | `.github/token-expiry.env` |

## 注意事项

- 上游历史会被 monorepo sync 重写，commit 无法直接追溯 → 上游对比一律按内容（文件 diff），不依赖 commit SHA 可达性。
- `.github/` 一律用本仓库 CI，不采用上游 workflow。
- Windows 编译不走 sccache（其 server 在巨型 crate 上会崩）。
- npm 包版本镜像 release tag（如 `0.2.117` ↔ `v0.2.117`）；打包 hotfix 用 `-fix.N` 后缀。
- 拿不准的隐私边界改动：停下来问，不要替用户做决定。

## 决策记录

人工维护：每次上游同步或本地改动，在此追加一行"原因/裁剪说明"，用 commit 或上游 SHA 作锚点。

### 本地改动记录（2026-08-03 起，含 fork 建立时回填）

| 提交 | 内容 | 原因/对应宗旨 |
|---|---|---|
| `2175302` | release 用专用 token | CI 安全：凭证最小化，避免权限过大 |
| `dac1977` `1b61150` | release notes 生成修复（macOS 校验和、shellcheck） | 发布流程可靠、可复现 |
| `903a6c6` | release 后发布预编译二进制到 npm | 分发渠道：npm 用户可全局安装（配合 `bfa46df` 文档） |
| `b062b23` | npm 包暴露 `grok` 命令 | npm 包可用性：装完即用 |
| `ede7b08` `f9f6455` | npm 版本/dist-tag 覆盖、资产版本与包版本解耦 | npm 发布灵活性：hotfix 不依赖新 release |
| `767f3a6` | token 到期日期登记与提醒 | CI 可维护：凭证轮换不靠记忆 |
| `d9fc97c` `777fbf4` `d62623a` | 容忍流式 web_search 缺 `action`/`query`；支持 DeepSeek 复数 `queries` | async-openai 临时镜像的兼容层（见「临时事项」） |
| `9d9e509` | npm registry E409 竞争重试、容忍 tombstone 版本 | npm 发布稳定性 |
| `bfa46df` | README 增加 npm/pnpm 全局安装说明 | 文档：配合 `903a6c6` 的分发渠道 |
| `chore: add script-driven upstream trace with CI auto-refresh`（本提交，SHA 见上方生成区） | 建立脚本驱动的上游 trace 机制：`scripts/upstream-trace.sh` 生成机械事实 + CI 在 push 到 main 后自动刷新回提交 + `.grok/skills/upstream-trace` 固化流程 + AGENTS.md 规则 | 宗旨「自持可跟进」：决策留痕、机械事实不靠人记；CI 接管刷新降低人工负担 |

### 上游同步记录（未来在此追加）

| 日期 | 上游 SHA | 处理（合入/裁剪/忽略） | 说明 |
|---|---|---|---|
| （待上游有更新时记录） | | | |
