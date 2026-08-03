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
