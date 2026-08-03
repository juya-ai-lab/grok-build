# JUYA-ISSUE-001：重复 model 值导致 api_backend 串台

## 状态卡片

- ID：JUYA-ISSUE-001
- Status：closed
- Resolution：fixed
- Created：2026-08-04
- Last updated：2026-08-04
- Affected version：xai-grok-shell 0.2.118
- Release tag：v0.2.118-fix1
- Next：无；后续仅跟踪上游同类变更

状态以 [juya-issue/README.md](../README.md) 的索引表为准；本卡片用于快速了解该 issue，不替代详情文档。

## 状态历史

| 日期 | 状态 | 记录 |
|---|---|---|
| 2026-08-04 | reported | 收到上游 issue，描述重复 model 值导致 api_backend 被后续区块覆盖。 |
| 2026-08-04 | investigating | 检查配置解析、slug propagation、sampler 配置和现有测试。 |
| 2026-08-04 | confirmed | 确认当前项目存在同类核心问题；issue 的“无条件最后区块胜出”表述需要收窄为“最后一个符合 donor 条件的 entry”。 |
| 2026-08-04 | resolution planned | 形成保留显式 api_backend provenance 的最小修复方案和回归测试方案；尚未修改实现。 |
| 2026-08-04 | fix_in_progress | 在 resolve_model_list 的 slug propagation 中保留显式 api_backend provenance，并加入同 slug 多 provider 回归测试。 |
| 2026-08-04 | verified | 326 个 config 测试、170 个 sampler 测试、9 个 api_backend 定向测试、格式检查和 cargo check 全部通过；完整 shell 测试另受无关 stack overflow/耗时限制影响，详见测试记录。 |
| 2026-08-04 | closed | 修复已提交并随 `v0.2.118-fix1` 发布；issue 记录转入 Closed。 |

## 详情

- [issue.md](issue.md)：上游 issue 原文。
- [investigation.md](investigation.md)：代码证据、触发条件、issue 准确性判断和本地验证。
- [solution-and-tests.md](solution-and-tests.md)：推荐修复、替代方案、测试方案和验收标准。
