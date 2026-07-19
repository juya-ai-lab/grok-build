//! Vendor-compat resolution for `grok inspect`.
//!
//! Resolves the local env/config/default stack into a diagnostic report.

use serde::Serialize;
use xai_grok_tools::types::compat::{COMPAT_CELLS, CompatCell, CompatConfig};

/// Derive the vendor origin from a file path. Returns `Some("cursor")` or
/// `Some("claude")` when the path passes through a vendor config directory;
/// `None` for native `.grok`/`.agents` paths.
pub(super) fn derive_vendor(path: &str) -> Option<&'static str> {
    if path.contains("/.cursor/") || path.contains("\\.cursor\\") || path.ends_with("/.cursor") {
        Some("cursor")
    } else if path.contains("/.claude/")
        || path.contains("\\.claude\\")
        || path.ends_with("/.claude")
        || path.contains("/.claude.json")
    {
        Some("claude")
    } else {
        None
    }
}

pub(super) fn instruction_compat_status(
    vendor: &Option<String>,
    file_type: &str,
    compat: &ExternalCompatReport,
) -> Option<CompatEntryStatus> {
    let surface = if file_type == "rules" {
        "rules"
    } else {
        "agents"
    };
    vendor_compat_status(vendor, surface, compat)
}

pub(super) fn vendor_compat_status(
    vendor: &Option<String>,
    surface: &str,
    compat: &ExternalCompatReport,
) -> Option<CompatEntryStatus> {
    let vendor = vendor.as_deref()?;
    if !matches!(vendor, "cursor" | "claude") {
        return None;
    }
    compat.status(vendor, surface)
}

/// Format a vendor tag for human output (e.g. " [cursor]"), empty for native.
pub(super) fn vendor_tag(vendor: &Option<String>) -> String {
    match vendor {
        Some(v) => format!(" [{}]", v),
        None => String::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CompatEntryStatus {
    Enabled,
    Disabled,
}

/// Which resolution layer determined a vendor-compat cell's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CompatSource {
    Env,
    Config,
    ConfigError,
    Default,
}

impl std::fmt::Display for CompatSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Env => "env",
            Self::Config => "config",
            Self::ConfigError => "config error; fail closed",
            Self::Default => "default",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCompatEntry {
    pub vendor: String,
    pub surface: String,
    pub enabled: bool,
    pub source: CompatSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCompatReport {
    pub remote_settings_loaded: bool,
    pub cells: Vec<ExternalCompatEntry>,
}

impl ExternalCompatReport {
    fn status(&self, vendor: &str, surface: &str) -> Option<CompatEntryStatus> {
        self.cells
            .iter()
            .find(|cell| cell.vendor == vendor && cell.surface == surface)
            .map(|cell| {
                if cell.enabled {
                    CompatEntryStatus::Enabled
                } else {
                    CompatEntryStatus::Disabled
                }
            })
    }
}

pub(super) fn resolve_inspect_compat(
    effective_config: Result<&toml::Value, ()>,
) -> ExternalCompatReport {
    resolve_inspect_compat_with_env(effective_config, |cell| {
        xai_grok_config::env_bool(cell.env_var())
    })
}

pub(super) fn resolve_inspect_compat_with_env(
    effective_config: Result<&toml::Value, ()>,
    env_value: impl Fn(CompatCell) -> Option<bool>,
) -> ExternalCompatReport {
    let defaults = CompatConfig::default();
    let cells = COMPAT_CELLS
        .into_iter()
        .filter(|cell| cell.is_runtime_supported())
        .map(|cell| {
            let config = crate::agent::config::compat_config_cell(effective_config, cell);
            resolve_compat_entry(cell, env_value(cell), config, defaults.value(cell))
        })
        .collect();

    ExternalCompatReport {
        remote_settings_loaded: false,
        cells,
    }
}

fn resolve_compat_entry(
    cell: CompatCell,
    env: Option<bool>,
    config: Result<Option<bool>, crate::agent::config::CompatConfigCellError>,
    default: bool,
) -> ExternalCompatEntry {
    let (config, config_error) = match config {
        Ok(value) => (value, false),
        Err(_) => (Some(false), true),
    };
    let resolved = crate::agent::config::resolve_compat_cell_with_env(env, config, None, default);
    let source = if env.is_some() {
        CompatSource::Env
    } else if config.is_some() {
        if config_error {
            CompatSource::ConfigError
        } else {
            CompatSource::Config
        }
    } else {
        CompatSource::Default
    };

    ExternalCompatEntry {
        vendor: cell.vendor().as_str().to_owned(),
        surface: cell.surface().as_str().to_owned(),
        enabled: resolved.value,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_reports_no_runtime_compatibility_cells() {
        let effective_config: toml::Value = toml::from_str(
            r#"
[compat.cursor]
skills = true
rules = true
agents = true
mcps = true
hooks = true
sessions = true
"#,
        )
        .unwrap();
        let report = resolve_inspect_compat_with_env(Ok(&effective_config), |_| Some(true));

        assert!(!report.remote_settings_loaded);
        assert!(report.cells.is_empty());
    }

    #[test]
    fn vendor_path_classification_remains_diagnostic_only() {
        assert_eq!(derive_vendor("/repo/.cursor/rules/a.md"), Some("cursor"));
        assert_eq!(derive_vendor("/repo/.claude/CLAUDE.md"), Some("claude"));
        assert_eq!(derive_vendor("/repo/.agents/skills/a/SKILL.md"), None);
    }
}
