# Upstream issue: duplicate model values clobber api_backend

以下内容保存用户提供的上游 issue 原文；API key 和第三方 URL 按原文保持脱敏。

## Summary

When multiple [model.*] sections in ~/.grok/config.toml declare the same model value, the api_backend of the last such section silently wins for all of them. A section configured with api_backend = "chat_completions" actually sends its request to {base_url}/responses, which then fails with:

~~~
Turn failed in 3.1s: Internal error: "serialization error: missing field sequence_number"
~~~

## Environment

- Grok Build version: 0.2.118
- OS: macOS (aarch64)
- Headless repro: grok -p "hi" -m <model>

## Reproduction

Config (API keys redacted; all three sections intentionally share model = "deepseek-v4-flash" because the respective gateways only accept that exact model id):

~~~toml
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]          # provider A: works with Responses API
model = "deepseek-v4-flash"
base_url = "https://api.deepseek.com"
api_backend = "responses"
api_key = "sk-***"

[model.deepseek-v4-flash-go]       # provider B: only works with Chat Completions
model = "deepseek-v4-flash"
base_url = "https://opencode.ai/zen/go/v1"
api_backend = "chat_completions"
api_key = "sk-***"

[model.deepseek-v4-flash-c]       # provider C: only works with Responses API
model = "deepseek-v4-flash"
base_url = "https://<third-provider>/v1"
api_backend = "responses"
api_key = "sk-***"
~~~

Steps:

1. grok -p "hi" -m deepseek-v4-flash-go
2. The request goes to https://opencode.ai/zen/go/v1/responses, not /chat/completions, and the turn fails with serialization error: missing field sequence_number in about 3 seconds.

## What actually happens

Debug logging (RUST_LOG=debug) shows the sampler config for deepseek-v4-flash-go resolves api_backend: Responses, the value of the last section deepseek-v4-flash-c, while base_url, api_key and context_window are correctly taken from the go section itself:

~~~
SamplerConfig { api_key: Some(...), base_url: "https://opencode.ai/zen/go/v1", model: "deepseek-v4-flash", ..., api_backend: Responses, ... }
...
Sending responses API stream request url=https://opencode.ai/zen/go/v1/responses method=POST
~~~

The request does use the correct section API key, so only api_backend is clobbered.

Control experiments (all headless, same config file, only the listed field changed):

| Experiment | Result for deepseek-v4-flash-go |
|---|---|
| Original config as above | 4/4 runs → /responses → serialization error |
| Append a 4th section with model = "deepseek-v4-flash" and api_backend = "chat_completions", now last | /chat/completions, turn succeeds |
| Change the third section api_backend to "chat_completions" | /chat/completions, turn succeeds |
| Move the go block to the top of the file | still /responses, still clobbered by the last section |

So the reported behavior is that the effective api_backend for a section is taken from the last section in file order that declares the same model value, regardless of that section's own setting or position.

## Why it surfaces as a serialization error

The gateway at https://opencode.ai/zen/go/v1 only implements Chat Completions correctly. Its /responses stream is non-conformant: events omit the mandatory sequence_number field, and sometimes the stream is only blank keepalives plus a response.completed with no output. Grok's ResponseStreamEvent struct requires sequence_number, so the deserializer fails:

~~~
Failed to deserialize ResponseStreamEvent from stream error=missing field sequence_number
raw_data={"id":"...","type":"response.output_text.delta","delta":"Hi","response":{"id":"...","model":"deepseek-v4-flash"}}
~~~

(Verified by direct curl to the endpoint; the same key works fine against /chat/completions on that gateway.)

## Expected behavior

Each [model.*] section should honor its own api_backend independently, even when several sections declare the same model value. At minimum, the section's explicit api_backend should win over a sibling section's value.

## Impact

Users cannot configure multiple providers that share a model id but require different API backends, such as one provider implementing Responses and another implementing only Chat Completions. The last section silently forces its backend on all of them and produces confusing serialization errors.

## Workaround

Keep all sections that share a model id on the same api_backend value, or use only one such section at a time by commenting out the others.
