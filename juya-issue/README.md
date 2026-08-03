# Issue 记录索引

本文件是 juya-issue 的轻量入口和状态索引。先看这里，再只打开与当前任务相关的 issue 子目录；不要为了处理一个 issue 扫描全部历史详情。

状态索引以本表为准；单个 issue 的 README 保存相同状态的上下文和变更历史。

## Active

| ID | 标题 | 状态 | 最后更新 | 下一步 |
|---|---|---|---|---|
暂无。

## Closed

| ID | 标题 | 状态 | 最后更新 | 发布 |
|---|---|---|---|---|
| [JUYA-ISSUE-001](001-duplicate-model-api-backend/) | 重复 model 值导致 api_backend 串台 | closed / resolution fixed | 2026-08-04 | `v0.2.118-fix1` |

## 状态定义

- reported：已记录，尚未完成调查。
- investigating：正在调查根因或复现条件。
- confirmed：问题已由代码、测试或可靠外部证据确认。
- fix_in_progress：修复已开始，但尚未完成验证。
- verified：修复和回归测试已通过，等待关闭记录。
- closed：问题已关闭，详情保留供历史追溯。
- rejected：调查后确认不是项目问题或不属于项目范围。
- blocked：已确认但需要外部状态或用户决定才能继续。

resolution 是独立字段，用于说明 confirmed 之后的处理状态，例如 planned、in_progress、fixed 或 not_planned。

## 目录约定

每个 issue 使用零填充的顺序编号和短标题目录，例如 001-duplicate-model-api-backend。目录至少包含：

- README.md：ID、状态、负责人/下一步、状态历史和文档导航；
- issue.md：原始 issue 或经过脱敏的完整描述；
- investigation.md：基于具体 commit 的调查结论和准确性判断；
- solution-and-tests.md：修复方案、取舍、测试方案和验收标准。

尚未实施的方案必须明确标记为 planned，不得写成已经存在的行为。
