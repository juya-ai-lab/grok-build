# 修复方案与测试方案

## 实施结果

方案已按最小改动原则落地：

- resolve_model_list 现在同时遍历 catalog key 和 entry；
- 通过 config model override 或其 model provider 的 provenance 判断 backend 是否显式配置；
- 显式配置 backend 的 entry（包括经 model provider 显式配置的 entry）不再接受 sibling backend 覆盖；
- 未显式配置 backend 的 entry 仍保留原有继承行为；
- context_window 的 sibling propagation 未改变。

实现文件：crates/codegen/xai-grok-shell/src/agent/config.rs。

## 验证结果

2026-08-04 的复核结果：

- `cargo fmt --package xai-grok-shell -- --check`：通过。
- `git diff --check`：通过。
- `cargo test -p xai-grok-shell --lib agent::config::tests -- --test-threads=1`：326/326 通过。
- `cargo test -p xai-grok-shell --lib api_backend -- --nocapture`：9/9 通过。
- `cargo test -p xai-grok-sampler --lib`：170/170 通过。
- `cargo check -p xai-grok-shell -p xai-grok-sampler`：通过。

一次完整 `xai-grok-shell` library 测试在默认线程栈下被无关的
`authenticated_401s_still_exhaust_after_three_retries` 测试 stack overflow
中止；该测试单独设置 `RUST_MIN_STACK=8388608` 后通过。随后尝试单线程完整运行
超过三分钟仍未完成，未将其作为本修复的通过证据，也没有观察到与本修复相关的新失败。

## 推荐修复

推荐做一个最小、局部的 provenance 修复：识别 config model key 或其 model provider 是否显式设置了 api_backend，并在 slug propagation 传播 backend 时跳过这些 key。

### 具体设计

在 slug propagation 循环中保留当前 catalog key，并查询该 key 的 ConfigModelOverride 及其 model provider：

~~~rust
let backend_is_explicit = cfg.config_models.get(key).is_some_and(|model_override| {
    model_override.api_backend.is_some()
        || model_override
            .model_provider
            .as_deref()
            .and_then(|provider_id| cfg.model_providers.get(provider_id))
            .is_some_and(|provider| provider.api_backend.is_some())
});
~~~

slug propagation 改为同时遍历 resolved 的 key 和 entry：

~~~rust
for (key, entry) in resolved.iter_mut() {
    // context_window 的 sibling propagation 保持现有行为。
    if entry.info.context_window.get() == default_cw {
        entry.info.context_window = donor_cw;
    }

    // 只有未显式配置 api_backend 的 config entry 才允许继承 backend。
    if !explicit_api_backend_keys.contains(key)
        && entry.info.api_backend == ApiBackend::default()
        && donor_backend != ApiBackend::default()
    {
        entry.info.api_backend = donor_backend;
    }
}
~~~

上面是设计示意，不是待直接复制的最终 patch；实际实现应遵守当前 crate 的 import、借用和日志风格。

### 为什么推荐这个方案

- 修复点正好位于丢失 provenance 的地方，改动面小。
- 显式写出的 responses、chat_completions、messages 都获得同样的“配置优先”保证。
- 未显式设置 backend 的 config entry 仍可从已有的远端/预取 catalog 或 sibling 继承，保留现有测试和兼容行为。
- context_window 的传播与 api_backend 的传播可以分开控制；不会因为修 backend 而意外改变 context_window 兼容逻辑。
- 不需要修改序列化格式，不需要迁移用户配置，也不触及隐私裁剪边界。

关键原则是：不能只继续比较 api_backend == default，因为那仍然无法区分显式 chat_completions 和未设置值。

## 可选方案及取舍

### 方案 A：完全删除 api_backend 的 sibling propagation

实现最简单，但可能破坏已有的 key/slug mismatch 兼容逻辑。现有测试 slug_propagation_inherits_api_backend_but_not_agent_type 明确把该继承行为当作当前契约，因此不推荐直接删除，除非产品决定 backend 永远不得跨 entry 继承。

### 方案 B：在 ModelEntry 或 ModelInfo 中加入 provenance 字段

例如记录 api_backend_explicit。这在多个解析层都需要判断显式性的情况下更稳健，但会扩大结构、序列化和测试面。当前问题只需要区分 [model.<id>] 是否显式设置 backend，先采用局部 key 集合更符合最小改动原则。

如果后续发现远端 catalog、managed config、prefetched model 等来源也需要精细区分显式性，再升级到统一 provenance 字段。

## 必须覆盖的回归测试

### 1. 三个同 slug entry 的显式 backend 独立性

新增配置 resolver 单元测试，例如：

~~~rust
#[test]
fn explicit_api_backend_is_not_overwritten_by_same_slug_sibling() {
    let raw = r#"
        [model.provider_a]
        model = "shared-model"
        base_url = "https://a.example/v1"
        api_key = "key-a"
        context_window = 300000
        api_backend = "responses"

        [model.provider_b]
        model = "shared-model"
        base_url = "https://b.example/v1"
        api_key = "key-b"
        context_window = 300000
        api_backend = "chat_completions"

        [model.provider_c]
        model = "shared-model"
        base_url = "https://c.example/v1"
        api_key = "key-c"
        context_window = 300000
        api_backend = "responses"
    "#;

    // Resolve the config and assert:
    // provider_a -> Responses
    // provider_b -> ChatCompletions
    // provider_c -> Responses
    // provider_b keeps its own base_url and api_key.
}
~~~

这个用例在当前实现上应能暴露 bug：provider_c 作为最后 donor 时，provider_b 的显式 chat_completions 会被改成 responses。

### 2. 采样配置继承检查

对 provider_b 调用现有 sampling_config_for_model 流程，断言 SamplerConfig.api_backend 仍是 ChatCompletions，且 model、base_url、credentials 没有串到 provider_a 或 provider_c。

这一步把 resolver 层的结果与实际请求层连接起来，防止只修了 catalog 状态而 sampler 仍读到错误值。

### 3. 既有兼容行为不回归

保留并继续运行：

- slug_propagation_inherits_api_backend_but_not_agent_type：未显式配置 backend 的 entry 仍可继承 sibling backend。
- slug_propagation_noop_when_no_donor：没有非默认 context_window donor 时不产生传播。
- slug_propagation_does_not_overwrite_explicit_context_window：显式 context_window 仍不被覆盖。

### 4. 顺序和 donor 条件控制

增加至少两个控制断言：

- 将 provider_b 移到第一个或最后一个，修复后结果都应由各自显式配置决定。
- 将所有 entry 的 context_window 改回 200000，传播层应 no-op；这用来区分 issue 的核心 bug 与其“无条件最后区块胜出”的过度概括。

可以将同一测试参数化为 responses → chat_completions → responses，以及 messages → chat_completions → responses，确认所有非默认 donor backend 都不会覆盖显式值。

### 5. endpoint 路由测试

使用本地 mock HTTP server 或现有 sampler 的 request-builder 测试，不访问真实第三方网关：

- ChatCompletions entry 必须请求 /chat/completions。
- Responses entry 必须请求 /responses。
- 多 provider 共用同一个 model slug 时，每个 entry 的 endpoint 仍与自己的 backend 对应。

这样可以验证 issue 中“错误 backend 导致错误 endpoint”的后果，同时避免 API key、网络和第三方响应格式给测试带来不确定性。

## 建议验证顺序

实现修复后先运行轻量、定向检查：

~~~text
cargo test -p xai-grok-shell --lib slug_propagation -- --nocapture
cargo test -p xai-grok-shell --lib parses_model_api_backend -- --nocapture
~~~

再运行涉及 sampler 路由的定向测试：

~~~text
cargo test -p xai-grok-sampler --lib
~~~

最后按仓库约定做受影响 crate 的编译检查：

~~~text
cargo check -p xai-grok-shell -p xai-grok-sampler
~~~

不需要为了这个配置修复执行全 workspace 全量测试或真实网关联调；若定向测试失败，再根据失败路径扩大范围。

## 验收标准

修复完成后必须同时满足：

1. 多个 [model.*] 区块共享同一 model 值时，显式 api_backend 各自生效。
2. 显式 chat_completions 不会被 responses 或 messages sibling 覆盖。
3. 未显式设置 api_backend 的 entry 仍保留既有继承兼容行为，除非产品另行决定移除该行为。
4. 每个 entry 的 base_url、api_key、model 和 api_backend 组合独立传递到 SamplerConfig。
5. 现有 slug propagation 测试与新增显式 backend 回归测试全部通过。
6. 仓库不产生真实凭据、外部请求、临时分支或测试残留。
