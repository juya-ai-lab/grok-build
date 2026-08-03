# 调查回应：重复 model 值导致 api_backend 串台

## 结论

当前项目存在上游 issue 所描述的核心问题，但 issue 把触发条件概括成了“最后一个区块无条件胜出”，这一点不完全准确。

更准确的结论是：

> 当多个配置 entry 的实际路由 model 相同，且其中存在非默认 context_window 的 sibling donor 时，后面的 donor 会按 model slug 覆盖同 slug entry 的默认 api_backend。由于 ChatCompletions 同时是默认值，程序无法区分“没有配置 api_backend”和“显式配置 api_backend = chat_completions”。

因此，显式写出的 chat_completions 可能被 sibling 的 responses 或 messages 覆盖。

## 检查范围和基准

- 仓库：juya-ai-lab/grok-build
- WSL 路径：/home/fantasty/code/juya-ai-lab/grok-build
- HEAD：4267b67a
- xai-grok-shell：0.2.118
- 调查时工作区：干净
- 未使用任何真实 API key，也未向 issue 中的第三方网关发请求。

## 实际数据流

### 1. 区块名称和实际 model 值是两套标识

解析器把 [model.<id>] 的区块名称放入 IndexMap，区块本身的 model 字段则保存为实际请求使用的 routing slug：

- 解析入口：[config_model_override_parse.rs](../../crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs#L207)
- 区块逐项插入 config_models：[config_model_override_parse.rs](../../crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs#L225)
- ConfigModelOverride 的 api_backend 保持为 Option：[config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L3970)

所以多个不同的区块名称可以合法地共享同一个 model 值。

### 2. 显式 api_backend 在第一阶段其实解析正确

ConfigModelOverride.apply 会把每个区块自己的 api_backend 写入当前 entry：

[config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L4022) 的 apply 流程在 [config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L4054) 处理显式 backend。

这说明问题不是 TOML parser 把不同区块直接合并，也不是一开始就把 API key 或 base_url 取错。

### 3. 后续 slug propagation 按实际 model 值聚合并丢失 provenance

resolve_model_list 的后处理创建如下逻辑等价的 donor map：

~~~rust
donors: HashMap<actual_model_slug, (context_window, api_backend)>
~~~

实现位置：[config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L3597)。

它只按 entry.info.model 做 key，不按 [model.<id>] 的区块 key 做 key。重复 slug 会在 HashMap 收集时互相覆盖；当前 xai-grok-shell 对 TOML 启用了 preserve_order，因此配置 entry 的遍历顺序与文件顺序相关：[Cargo.toml](../../crates/codegen/xai-grok-shell/Cargo.toml#L65)。

随后，对同 slug entry 执行：

~~~rust
if entry.info.api_backend == ApiBackend::default()
    && donor_backend != ApiBackend::default()
{
    entry.info.api_backend = donor_backend;
}
~~~

对应源码：[config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L3610)。

ApiBackend 的默认值就是 ChatCompletions：[types.rs](../../crates/codegen/xai-grok-sampling-types/src/types.rs#L1010)。因此显式 chat_completions 和未配置 backend 在这一阶段看起来完全一样。

### 4. 错误 backend 会传递到 sampler

sampling_config_for_model 直接从 ModelInfo 复制 api_backend 到 SamplerConfig：[config.rs](../../crates/codegen/xai-grok-shell/src/agent/config.rs#L5122)。

所以一旦 slug propagation 把 go entry 改成 Responses，后续请求就会按照 Responses API 的路径和流格式处理；第三方网关只实现 Chat Completions 时，出现 sequence_number 反序列化错误是合理的下游表现。但本次调查没有访问该外部网关，因此不对具体网关响应作独立验证。

## 与 issue 原文的对应关系

| issue 说法 | 当前代码判断 |
|---|---|
| 不同 [model.*] 区块共享同一个 model 值会产生串台 | 核心属实 |
| 显式 chat_completions 可能被 responses 覆盖 | 属实 |
| base_url 和 api_key 仍来自自己的区块 | 代码上成立；donor tuple 只携带 context_window 和 api_backend |
| 最后一个同 slug 区块无条件胜出 | 过宽；准确说是最后一个符合 donor 条件的 entry 胜出 |
| 只要配置片段没有 context_window 就必现 | 不一定；如果所有 entry 的 context_window 都是 200000 且没有远端/预取 donor，这层传播会 no-op |
| 错误最终表现为 Responses stream 的 sequence_number 错误 | 对支持不完整的第三方 Responses endpoint 来说合理，但本次未做外部网络验证 |

“最后一个”之所以在典型运行中成立，是因为：

1. TOML preserve_order 保留了配置顺序；
2. resolved 按该顺序处理配置 entry；
3. duplicate model slug 收集到 HashMap 时后一个值替换前一个值。

但如果最后一个同 slug entry 使用默认 chat_completions，或者它的 context_window 仍是默认值，它不一定会成为有效 donor。

## 本地验证

运行命令：

~~~text
cargo test -p xai-grok-shell --lib slug_propagation -- --nocapture
~~~

结果：4 passed, 0 failed。通过的测试包括：

- slug_propagation_noop_when_no_donor
- slug_propagation_enterprise_managed_config_key_mismatch
- slug_propagation_does_not_overwrite_explicit_context_window
- slug_propagation_inherits_api_backend_but_not_agent_type

最后一个现有测试证明 sibling backend propagation 是当前设计的一部分，但它只测试“默认 backend 从 sibling 继承”，没有测试“显式 chat_completions 不应被继承覆盖”。

## 调查判断

这是一个真实的配置 provenance 丢失问题，不是 issue 把普通 API 错误误认成配置问题。根因在 backend 传播条件使用了 resolved enum 的默认值来推断“是否显式配置”，而没有保留用户配置的显式性。
