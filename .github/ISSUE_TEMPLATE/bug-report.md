---
name: Bug report / compatibility issue
about: Report a reproducible bug and identify whether it belongs to upstream or this fork
title: "[Bug] "
labels: ""
assignees: ""
---

## Scope / 问题归属

请选择至少一项：

- [ ] `upstream-originated`：问题来自上游代码或上游版本
- [ ] `fork-specific`：问题只存在于本 fork 的改动、裁剪或发布流程
- [ ] `both`：上游和本 fork 都受影响，但处理方式可能不同
- [ ] `unknown`：尚未确定，调查后补充

## Affected artifact / 受影响产物

如果问题涉及实际运行或讨论中的产物，请勾选适用项；这不是所有 issue 的必填分类。若问题
不涉及产物，或暂时无法判断，请选择对应选项。npm 与 pnpm 是同一个 JS wrapper 的两种安装
方式，不单独区分。由于 npm package 通常分发或调用同一平台产物，下面的 npm package 默认
勾选；如果确认问题只影响直接平台产物，或不涉及 npm，请取消它。

### User-facing products

- [ ] Direct platform artifact — `grok-<version>-linux-x86_64`
- [ ] Direct platform artifact — `grok-<version>-linux-aarch64`
- [ ] Direct platform artifact — `grok-<version>-macos-x86_64`
- [ ] Direct platform artifact — `grok-<version>-macos-aarch64`
- [ ] Direct platform artifact — `grok-<version>-windows-x86_64.exe`
- [ ] Direct platform artifact — `grok-<version>-windows-aarch64.exe`
- [x] npm package：`@juya-ai-lab/grok-build`（npm/pnpm 安装的 JS wrapper；默认通常与平台产物共同受影响）

### Source build and other

- [ ] Source build（例如 `cargo build` 或本地 checkout）
- [ ] Other / unknown：
- [ ] Not artifact-related / 不涉及产物
- [ ] Not yet determined / 尚未确定

Artifact filename, package version, or other identifier:

```text
artifact:
version/tag:
```

## Version and commit provenance / 版本与提交来源

| | Upstream | This fork |
|---|---|---|
| Version / tag |  |  |
| Commit |  |  |
| Commit URL |  |  |

如果问题来自上游，请填写上游版本和 commit 链接；如果在本 fork 复现，也填写本 fork 实际运行的版本、tag 或 commit。不要只写“当前版本”。

## Summary / 问题摘要

<!-- 用一两句话说明用户看到的现象和影响。 -->

## Reproduction / 复现

<!-- 写出环境、配置、命令和最小复现步骤；API key、token、工作区内容必须脱敏。 -->

```text
environment:
command:
steps:
```

## Expected vs actual / 预期与实际

### Expected

<!-- 预期行为。 -->

### Actual

<!-- 实际行为、错误信息或日志。 -->

## Evidence and investigation / 证据与调查

<!-- 附上源码路径、相关 commit、日志、测试结果和对根因的判断。区分已证实、推测和未验证内容。 -->

## Validation status / 验证状态

- [ ] 仅报告了复现现象
- [ ] 已完成源码/单元测试验证
- [ ] 已完成编译检查
- [ ] 已使用实际编译产物 smoke test
- [ ] 已记录发布产物版本、命令和 endpoint/行为证据
- [ ] 可以关闭 issue

Issue 只有在关闭条件对应的实际验证完成后才应关闭；源码测试通过不自动等同于编译产物验证通过。

## Privacy / 隐私

请勿粘贴真实 API key、token、工作区内容、提示词或其他敏感数据。使用最小脱敏配置和可公开的日志片段。
