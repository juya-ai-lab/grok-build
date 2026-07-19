//! Plugin hooks adapter — pre-filter and source-entry builder.
//!
//! This module is a bridge between plugin hook JSON files and the shared
//! `xai-grok-hooks` runtime.  It pre-filters unsupported events from plugin
//! hook files before passing them to `parse_hook_file()`, and injects
//! plugin-specific environment variables into the resulting `HookSpec` entries.
//!
//! This is NOT a second hooks engine — it feeds into the existing
//! `xai-grok-hooks` crate's parser and runtime.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xai_grok_hooks::config::{HookSpec, parse_hook_file};

use super::manifest::substitute_env_vars;

/// Supported hook event names.
/// Both PascalCase and snake_case forms are accepted.
const SUPPORTED_EVENTS: &[&str] = &[
    // v0 events — PascalCase and snake_case
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "SessionEnd",
    "session_start",
    "pre_tool_use",
    "post_tool_use",
    "session_end",
    // v2 events — PascalCase and snake_case
    "Notification",
    "Stop",
    "UserPromptSubmit",
    "SubagentStart",
    "SubagentEnd",
    "notification",
    "stop",
    "user_prompt_submit",
    "subagent_start",
    "subagent_end",
];

fn is_disabled_vendor_env(name: &str) -> bool {
    starts_with_ignore_ascii_case(name, "CLAUDE_")
        || starts_with_ignore_ascii_case(name, "CODEX_")
        || starts_with_ignore_ascii_case(name, "GROK_CLAUDE_")
        || starts_with_ignore_ascii_case(name, "GROK_CODEX_")
}

fn starts_with_ignore_ascii_case(name: &str, prefix: &str) -> bool {
    name.as_bytes()
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix.as_bytes()))
}

fn expand_plugin_value(source: &str, plugin_root: &str, plugin_data: &str) -> String {
    let substituted = substitute_env_vars(source, plugin_root, plugin_data);
    expand_non_vendor_env(&substituted)
}

/// Expand ordinary environment references while preserving every Claude/Codex
/// signal literally. Splitting around disabled references avoids a sentinel
/// that could collide with adversarial input or an expanded ordinary value.
fn expand_non_vendor_env(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0;
    let mut pos = 0;

    while pos < bytes.len() {
        if bytes[pos] != b'$' || pos + 1 >= bytes.len() {
            pos += 1;
            continue;
        }

        let (name_start, name_end, reference_end) = if bytes[pos + 1] == b'{' {
            let name_start = pos + 2;
            let mut name_end = name_start;
            while name_end < bytes.len()
                && (bytes[name_end].is_ascii_alphanumeric() || bytes[name_end] == b'_')
            {
                name_end += 1;
            }
            let Some(close_offset) = input[name_end..].find('}') else {
                pos += 1;
                continue;
            };
            (name_start, name_end, name_end + close_offset + 1)
        } else if bytes[pos + 1].is_ascii_alphabetic() || bytes[pos + 1] == b'_' {
            let name_start = pos + 1;
            let mut name_end = name_start + 1;
            while name_end < bytes.len()
                && (bytes[name_end].is_ascii_alphanumeric() || bytes[name_end] == b'_')
            {
                name_end += 1;
            }
            (name_start, name_end, name_end)
        } else {
            pos += 1;
            continue;
        };

        let name = &input[name_start..name_end];
        if name.is_empty() || !is_disabled_vendor_env(name) {
            pos += 1;
            continue;
        }

        out.push_str(&xai_grok_config::expand_env_vars_in_string(
            &input[cursor..pos],
        ));
        out.push_str(&input[pos..reference_end]);
        cursor = reference_end;
        pos = reference_end;
    }

    out.push_str(&xai_grok_config::expand_env_vars_in_string(
        &input[cursor..],
    ));
    out
}

/// Parse plugin hook files with pre-filtering and env injection.
///
/// For each trusted plugin with hooks, this function:
/// 1. Reads the hooks JSON file
/// 2. Pre-filters unsupported event names (avoiding parse failures)
/// 3. Parses via `parse_hook_file()`
/// 4. Injects plugin-specific env vars into each resulting `HookSpec`
///
/// Returns `(specs, warnings)` — specs are ready to merge into the
/// `HookRegistry`, warnings are unsupported-handler or parse errors.
pub fn parse_plugin_hooks(
    hooks_path: &Path,
    plugin_name: &str,
    plugin_root: &str,
    plugin_data: &str,
) -> (Vec<HookSpec>, Vec<String>) {
    let Some(hooks_path) = xai_grok_config::validate_grok_path(hooks_path) else {
        return (
            vec![],
            vec![format!(
                "plugin {plugin_name}: refusing hooks file under Claude/Codex vendor state"
            )],
        );
    };
    let content = match std::fs::read_to_string(&hooks_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                vec![],
                vec![format!(
                    "plugin {plugin_name}: failed to read hooks file {}: {e}",
                    hooks_path.display()
                )],
            );
        }
    };

    let (specs, warnings) =
        process_hooks_content(&content, &hooks_path, plugin_name, plugin_root, plugin_data);
    tracing::debug!(
        plugin = plugin_name,
        hooks_count = specs.len(),
        warnings = warnings.len(),
        "plugin hooks loaded from file"
    );
    (specs, warnings)
}

/// Parse inline hooks from a manifest JSON value.
///
/// Same pipeline as [`parse_plugin_hooks()`] but skips the file I/O step.
/// The `value` is expected to be the manifest's inline hooks object,
/// structured as `{ "hooks": { "EventName": [...] } }`.
pub fn parse_plugin_hooks_from_value(
    value: &serde_json::Value,
    plugin_name: &str,
    plugin_root: &str,
    plugin_data: &str,
) -> (Vec<HookSpec>, Vec<String>) {
    let content = serde_json::to_string(value).unwrap_or_default();
    // Use a synthetic path for parse_hook_file's source_dir (resolves relative commands).
    let synthetic_path = Path::new(plugin_root).join("plugin.json");
    let (specs, warnings) = process_hooks_content(
        &content,
        &synthetic_path,
        plugin_name,
        plugin_root,
        plugin_data,
    );
    tracing::debug!(
        plugin = plugin_name,
        hooks_count = specs.len(),
        warnings = warnings.len(),
        "plugin hooks loaded from manifest inline"
    );
    (specs, warnings)
}

/// Shared processing pipeline for plugin hooks (file-based or inline).
///
/// Pre-filters unsupported events, parses via `parse_hook_file()`,
/// injects plugin env vars, and namespaces hook names.
fn process_hooks_content(
    content: &str,
    source_path: &Path,
    plugin_name: &str,
    plugin_root: &str,
    plugin_data: &str,
) -> (Vec<HookSpec>, Vec<String>) {
    let (filtered_content, skipped_events) = prefilter_unsupported_events(content);
    let mut warnings: Vec<String> = Vec::new();

    for event in &skipped_events {
        tracing::info!(
            plugin = plugin_name,
            event = event,
            "skipping unsupported hook event from plugin"
        );
        warnings.push(format!(
            "plugin {plugin_name}: skipped unsupported event '{event}'"
        ));
    }

    let (mut specs, parse_errors) = parse_hook_file(&filtered_content, source_path);

    for err in &parse_errors {
        let msg = format!("plugin {plugin_name}: {err}");
        tracing::warn!("{msg}");
        warnings.push(msg);
    }

    // Build the native plugin environment. Claude/Codex compatibility signals
    // are deliberately not injected, regardless of where discovery happened.
    let plugin_env: HashMap<String, String> = HashMap::from([
        ("GROK_PLUGIN_ROOT".to_string(), plugin_root.to_string()),
        ("GROK_PLUGIN_DATA".to_string(), plugin_data.to_string()),
    ]);

    // Inject env vars and update source labels.
    //
    // The plugin adapter owns the `GROK_PLUGIN_*` keys, so plugin-injected
    // values must always win over any
    // user-declared `env` on the hook JSON for those specific keys --
    // otherwise a plugin author could (deliberately or by accident) pin
    // the plugin root to an arbitrary path and break the plugin
    // contract. User-declared keys not owned by the plugin are
    // preserved.
    for spec in &mut specs {
        spec.extra_env.retain(|key, _| !is_disabled_vendor_env(key));
        for (k, v) in &plugin_env {
            spec.extra_env.insert(k.clone(), v.clone());
        }
        // Prefix name with plugin namespace for identification
        spec.name = format!("plugin/{}/{}", plugin_name, spec.name);
        // Substitute native plugin env vars in command paths at config-load
        // time, regardless of which spawn branch the runner takes.
        if let Some(command_source) = spec.command_raw.clone().or_else(|| {
            spec.command
                .as_ref()
                .map(|cmd| cmd.to_string_lossy().into_owned())
        }) {
            // First substitute Grok's plugin-specific placeholders, then run
            // the result through generic env expansion while protecting every
            // Claude/Codex environment signal from compatibility substitution.
            let expanded = expand_plugin_value(&command_source, plugin_root, plugin_data);
            spec.command = Some(PathBuf::from(expanded));
        }
        if let Some(url_source) = spec.url_raw.clone().or_else(|| spec.url.clone()) {
            spec.url = Some(expand_plugin_value(&url_source, plugin_root, plugin_data));
        }
    }

    (specs, warnings)
}

/// Pre-filter unsupported event names from a hooks JSON file.
///
/// Parses the JSON, removes event keys from the `"hooks"` object that are
/// not in the supported set, and returns the filtered JSON string plus the
/// list of removed event names.
///
/// This is critical because the hooks crate uses `HashMap<HookEventName, ...>`
/// deserialization which causes a full parse failure on unknown event names.
fn prefilter_unsupported_events(json_content: &str) -> (String, Vec<String>) {
    let mut value: serde_json::Value = match serde_json::from_str(json_content) {
        Ok(v) => v,
        Err(_) => {
            // If JSON is invalid, return as-is and let parse_hook_file handle the error
            return (json_content.to_string(), vec![]);
        }
    };

    let mut skipped = Vec::new();

    if let Some(hooks_obj) = value.get_mut("hooks").and_then(|v| v.as_object_mut()) {
        let keys_to_remove: Vec<String> = hooks_obj
            .keys()
            .filter(|key| !SUPPORTED_EVENTS.contains(&key.as_str()))
            .cloned()
            .collect();

        for key in keys_to_remove {
            hooks_obj.remove(&key);
            skipped.push(key);
        }
    }

    (
        serde_json::to_string(&value).unwrap_or_else(|_| json_content.to_string()),
        skipped,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilter_removes_unsupported_events() {
        let json = r#"{
            "hooks": {
                "SessionStart": [{"hooks": [{"type": "command", "command": "echo start"}]}],
                "CustomEvent": [{"hooks": [{"type": "command", "command": "echo custom"}]}],
                "UnknownHook": [{"hooks": [{"type": "command", "command": "echo unknown"}]}],
                "PostToolUse": [{"hooks": [{"type": "command", "command": "echo post"}]}]
            }
        }"#;

        let (filtered, skipped) = prefilter_unsupported_events(json);

        assert_eq!(skipped.len(), 2);
        assert!(skipped.contains(&"CustomEvent".to_string()));
        assert!(skipped.contains(&"UnknownHook".to_string()));

        let parsed: serde_json::Value = serde_json::from_str(&filtered).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();
        assert!(hooks.contains_key("SessionStart"));
        assert!(hooks.contains_key("PostToolUse"));
        assert!(!hooks.contains_key("CustomEvent"));
        assert!(!hooks.contains_key("UnknownHook"));
    }

    #[test]
    fn prefilter_preserves_all_supported_events() {
        let json = r#"{
            "hooks": {
                "SessionStart": [],
                "PreToolUse": [],
                "PostToolUse": [],
                "SessionEnd": []
            }
        }"#;

        let (_, skipped) = prefilter_unsupported_events(json);
        assert!(skipped.is_empty());
    }

    #[test]
    fn prefilter_handles_snake_case_events() {
        let json = r#"{
            "hooks": {
                "session_start": [],
                "pre_tool_use": [],
                "unknown_event": []
            }
        }"#;

        let (_, skipped) = prefilter_unsupported_events(json);
        assert_eq!(skipped.len(), 1);
        assert!(skipped.contains(&"unknown_event".to_string()));
    }

    #[test]
    fn prefilter_handles_invalid_json() {
        let json = "not valid json{";
        let (filtered, skipped) = prefilter_unsupported_events(json);
        assert_eq!(filtered, json); // returned as-is
        assert!(skipped.is_empty());
    }

    #[test]
    fn prefilter_handles_no_hooks_key() {
        let json = r#"{"settings": {}}"#;
        let (_, skipped) = prefilter_unsupported_events(json);
        assert!(skipped.is_empty());
    }

    #[test]
    fn parse_plugin_hooks_from_file() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();

        let hooks_file = hooks_dir.join("hooks.json");
        std::fs::write(
            &hooks_file,
            r#"{
                "hooks": {
                    "SessionStart": [
                        {
                            "hooks": [
                                {"type": "command", "command": "echo plugin-hook"}
                            ]
                        }
                    ],
                    "FutureEvent": [
                        {
                            "hooks": [
                                {"type": "command", "command": "echo unsupported"}
                            ]
                        }
                    ]
                }
            }"#,
        )
        .unwrap();

        let (specs, warnings) =
            parse_plugin_hooks(&hooks_file, "my-plugin", "/path/to/plugin", "/path/to/data");

        // Should have 1 spec from SessionStart, FutureEvent was filtered
        assert_eq!(specs.len(), 1);
        assert!(specs[0].name.starts_with("plugin/my-plugin/"));
        assert_eq!(
            specs[0].extra_env.get("GROK_PLUGIN_ROOT").unwrap(),
            "/path/to/plugin"
        );
        assert!(!specs[0].extra_env.contains_key("CLAUDE_PLUGIN_ROOT"));
        assert_eq!(
            specs[0].extra_env.get("GROK_PLUGIN_DATA").unwrap(),
            "/path/to/data"
        );

        // Should have a warning about FutureEvent
        assert!(warnings.iter().any(|w| w.contains("FutureEvent")));
    }

    #[cfg(unix)]
    #[test]
    fn parse_plugin_hooks_revalidates_path_after_discovery() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let discovered_path = tmp.path().join("ordinary-plugin/hooks/hooks.json");
        std::fs::create_dir_all(discovered_path.parent().unwrap()).unwrap();
        std::fs::write(&discovered_path, r#"{"hooks":{}}"#).unwrap();

        let vendor_path = tmp.path().join(".claude/plugins/leak/hooks.json");
        std::fs::create_dir_all(vendor_path.parent().unwrap()).unwrap();
        std::fs::write(
            &vendor_path,
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"leak"}]}]}}"#,
        )
        .unwrap();
        std::fs::remove_file(&discovered_path).unwrap();
        symlink(&vendor_path, &discovered_path).unwrap();

        let (specs, warnings) = parse_plugin_hooks(&discovered_path, "swapped", "/plugin", "/data");
        assert!(specs.is_empty());
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("vendor state"))
        );
    }

    #[test]
    fn parse_inline_hooks_from_value() {
        let value = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {
                        "hooks": [
                            {"type": "command", "command": "echo inline-hook"}
                        ]
                    }
                ]
            }
        });

        let (specs, warnings) = parse_plugin_hooks_from_value(
            &value,
            "inline-plugin",
            "/path/to/plugin",
            "/path/to/data",
        );

        assert_eq!(specs.len(), 1);
        assert!(specs[0].name.starts_with("plugin/inline-plugin/"));
        assert_eq!(
            specs[0].extra_env.get("GROK_PLUGIN_ROOT").unwrap(),
            "/path/to/plugin"
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_inline_hooks_filters_unsupported_events() {
        let value = serde_json::json!({
            "hooks": {
                "PostToolUse": [
                    {"hooks": [{"type": "command", "command": "echo post"}]}
                ],
                "FutureEvent": [
                    {"hooks": [{"type": "command", "command": "echo future"}]}
                ]
            }
        });

        let (specs, warnings) =
            parse_plugin_hooks_from_value(&value, "filter-test", "/root", "/data");

        // PostToolUse is supported, FutureEvent is not
        assert_eq!(specs.len(), 1);
        assert!(warnings.iter().any(|w| w.contains("FutureEvent")));
    }

    /// Grok plugin tokens are substituted at config-load time, while Claude
    /// Code tokens remain literal and receive no compatibility treatment.
    #[test]
    fn parse_plugin_hooks_substitutes_plugin_root_in_command() {
        let value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [
                        {"type": "command", "command": "${CLAUDE_PLUGIN_ROOT}/hooks/pre.sh"},
                        {"type": "command", "command": "${GROK_PLUGIN_ROOT}/hooks/alias.sh"},
                        {"type": "command", "command": "${CLAUDE_PLUGIN_DATA}/cache/post.sh"}
                    ]}
                ]
            }
        });

        let (specs, warnings) = parse_plugin_hooks_from_value(
            &value,
            "gb1183-plugin",
            "/opt/plugins/gb1183",
            "/var/plugins/gb1183",
        );

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(specs.len(), 3);

        let commands: Vec<String> = specs
            .iter()
            .map(|s| s.command.as_ref().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(commands.contains(&"${CLAUDE_PLUGIN_ROOT}/hooks/pre.sh".to_string()));
        assert!(commands.contains(&"/opt/plugins/gb1183/hooks/alias.sh".to_string()));
        assert!(commands.contains(&"${CLAUDE_PLUGIN_DATA}/cache/post.sh".to_string()));

        // The plugin adapter must NOT mutate
        // `command_raw`. The pager UI / ACP DTO surface the raw form
        // for display so users see what they wrote (and so any secrets
        // resolved from `extra_env` don't leak). A future "tidy" pass
        // that mistakenly rewrote `command_raw` would silently break
        // the secrets-leakage protection.
        let raws: Vec<&str> = specs
            .iter()
            .map(|s| s.command_raw.as_deref().unwrap_or(""))
            .collect();
        assert!(
            raws.contains(&"${CLAUDE_PLUGIN_ROOT}/hooks/pre.sh"),
            "command_raw must preserve the source string verbatim, got {raws:?}"
        );
        assert!(
            raws.contains(&"${GROK_PLUGIN_ROOT}/hooks/alias.sh"),
            "command_raw must preserve the source string verbatim, got {raws:?}"
        );
        assert!(
            raws.contains(&"${CLAUDE_PLUGIN_DATA}/cache/post.sh"),
            "command_raw must preserve the source string verbatim, got {raws:?}"
        );
    }

    #[test]
    fn parse_inline_hooks_handles_empty_value() {
        let value = serde_json::json!({});
        let (specs, warnings) = parse_plugin_hooks_from_value(&value, "empty", "/root", "/data");
        assert!(specs.is_empty());
        assert!(warnings.is_empty());
    }

    /// Regression: plugin hook commands that reference generic env vars
    /// (e.g. `${HOME}` / `$HOME`) must be expanded at config-load time
    /// just like managed MCP server commands. Otherwise resolution
    /// depends on the runtime `sh -c` heuristic in
    /// `xai-grok-hooks::runner::command`, which can fail for hooks
    /// whose handler doesn't otherwise contain shell metacharacters.
    /// Plugin hooks must not be double-expanded: a `${GROK_PLUGIN_ROOT}`
    /// reference resolves to the plugin root exactly once, and the result
    /// contains no leftover `$` placeholders. This is the contract the
    /// hooks_adapter has long held, and it must continue to hold
    /// now that `parse_hook_file` itself does an env-expansion pass with
    /// the per-hook `extra_env`. The first pass (in `parse_hook_file`)
    /// runs against an EMPTY `extra_env` for plugin hooks (the adapter
    /// only fills it in afterwards), so the placeholder survives that
    /// pass and the second pass (here, after `extra_env` is wired in)
    /// resolves it.
    #[test]
    fn parse_plugin_hooks_resolves_plugin_root_exactly_once() {
        let value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [
                        {"type": "command", "command": "${GROK_PLUGIN_ROOT}/x.sh"}
                    ]}
                ]
            }
        });

        let (specs, warnings) = parse_plugin_hooks_from_value(
            &value,
            "no-double-expand",
            "/the/plugin/root",
            "/the/plugin/data",
        );

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(specs.len(), 1);
        let cmd = specs[0]
            .command
            .as_ref()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(cmd, "/the/plugin/root/x.sh");
        assert!(
            !cmd.contains('$'),
            "command must not contain leftover $: {cmd}"
        );
    }

    /// Plugin hook JSON may declare its own `env` map. The user-declared
    /// keys land in `extra_env`, but the plugin adapter MUST override
    /// any user-declared value for keys the plugin owns
    /// (`GROK_PLUGIN_ROOT` and `GROK_PLUGIN_DATA`). Disabled Claude/Codex
    /// compatibility keys are removed, while unrelated user env is preserved.
    #[test]
    fn parse_plugin_hooks_user_env_merged_with_plugin_precedence() {
        // Claude/Codex compatibility keys must not survive the adapter,
        // including mixed-case spellings and Grok-prefixed aliases.
        let value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [
                        {
                            "type": "command",
                            "command": "echo hi",
                            "env": {
                                "FOO": "bar",
                                "CLAUDE_PLUGIN_ROOT": "/user/wins?",
                                "cLaUdE_SESSION": "user-session",
                                "CoDeX_HoMe": "/user/codex",
                                "gRoK_cLaUdE_SESSION": "grok-claude-session",
                                "GrOk_CoDeX_TOKEN": "grok-codex-token",
                                "GROK_PLUGIN_ROOT": "/user/wins?",
                                "CLAUDE_PLUGIN_DATA": "/user/wins?",
                                "GROK_PLUGIN_DATA": "/user/wins?"
                            }
                        }
                    ]}
                ]
            }
        });

        let (specs, warnings) = parse_plugin_hooks_from_value(
            &value,
            "user-env-plugin",
            "/actual/plugin/root",
            "/actual/plugin/data",
        );

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(specs.len(), 1);

        // User-declared key the plugin doesn't own: preserved verbatim.
        assert_eq!(
            specs[0].extra_env.get("FOO").map(String::as_str),
            Some("bar"),
            "user-declared env keys must survive plugin merge"
        );

        // Native plugin-owned keys override user values.
        for (key, expected) in [
            ("GROK_PLUGIN_ROOT", "/actual/plugin/root"),
            ("GROK_PLUGIN_DATA", "/actual/plugin/data"),
        ] {
            assert_eq!(
                specs[0].extra_env.get(key).map(String::as_str),
                Some(expected),
                "plugin-injected key {key} must override user-declared value"
            );
        }
        for key in [
            "CLAUDE_PLUGIN_ROOT",
            "CLAUDE_PLUGIN_DATA",
            "cLaUdE_SESSION",
            "CoDeX_HoMe",
            "gRoK_cLaUdE_SESSION",
            "GrOk_CoDeX_TOKEN",
        ] {
            assert!(
                !specs[0].extra_env.contains_key(key),
                "disabled vendor key survived plugin env filtering: {key}"
            );
        }
    }

    #[test]
    fn plugin_value_expansion_preserves_vendor_signals_case_insensitively() {
        let claude_key = "cLaUdE_HOOKS_ADAPTER_TEMPLATE_TEST";
        let codex_key = "gRoK_cOdEx_HOOKS_ADAPTER_TEMPLATE_TEST";
        let ordinary_key = "GROK_HOOKS_ADAPTER_ORDINARY_TEMPLATE_TEST";
        // SAFETY: all keys are unique to this test and restored before asserts.
        unsafe {
            std::env::set_var(claude_key, "claude-leak");
            std::env::set_var(codex_key, "codex-leak");
            std::env::set_var(ordinary_key, "ordinary-value");
        }

        let source = format!("${{{claude_key}}}:${codex_key}:${{{ordinary_key}}}");
        let expanded = expand_plugin_value(&source, "/plugin", "/data");

        // SAFETY: restore the unique test keys before making assertions.
        unsafe {
            std::env::remove_var(claude_key);
            std::env::remove_var(codex_key);
            std::env::remove_var(ordinary_key);
        }

        assert_eq!(
            expanded,
            format!("${{{claude_key}}}:${codex_key}:ordinary-value")
        );
    }

    #[test]
    fn parse_plugin_hooks_expands_generic_env_vars_in_command() {
        // SAFETY: only mutated within this single-threaded test.
        // SAFETY: this test sets process env vars; tokio test macros
        // serialize tests within the same module by default but to be
        // robust use a uniquely-named var.
        let var = "GB1183_HOOKS_ADAPTER_TEST_HOME";
        // SAFETY: env writes are not thread-safe; this test is single-threaded.
        unsafe {
            std::env::set_var(var, "/expanded/home");
        }

        let cmd_braces = format!("${{{var}}}/helper.sh");
        let cmd_bare = format!("${var}/raw.sh");
        let value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [
                        {"type": "command", "command": cmd_braces},
                        {"type": "command", "command": cmd_bare},
                    ]}
                ]
            }
        });

        let (specs, warnings) =
            parse_plugin_hooks_from_value(&value, "env-expand", "/root", "/data");

        // SAFETY: env writes are not thread-safe; this test is single-threaded.
        unsafe {
            std::env::remove_var(var);
        }

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(specs.len(), 2);

        let commands: Vec<String> = specs
            .iter()
            .map(|s| s.command.as_ref().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(
            commands.contains(&"/expanded/home/helper.sh".to_string()),
            "missing brace-form expansion: {commands:?}"
        );
        assert!(
            commands.contains(&"/expanded/home/raw.sh".to_string()),
            "missing bare-form expansion: {commands:?}"
        );
        for cmd in &commands {
            assert!(!cmd.contains('$'), "command still contains $: {cmd}");
        }
    }
}
