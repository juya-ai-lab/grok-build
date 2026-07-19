//! Legacy Claude marketplace compatibility shims.
//!
//! Claude marketplace discovery is disabled. The public entry points remain
//! temporarily available for downstream callers, but fail closed without
//! reading the filesystem.

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResolvedMarketplace {
    pub name: String,
    pub path: PathBuf,
    pub plugin_dirs: Vec<PathBuf>,
}

/// Disabled compatibility entry point. Does not inspect `git_root` or any
/// `.claude/settings.json` file.
pub fn resolve(_git_root: &Path) -> Vec<ResolvedMarketplace> {
    Vec::new()
}

/// Disabled compatibility entry point. Does not inspect the JSON value.
pub fn parse_enabled_disabled_plugins(_json: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    (Vec::new(), Vec::new())
}

/// Disabled compatibility entry point. Does not read `path`.
pub fn load_enabled_disabled_plugins(_path: &Path) -> (Vec<String>, Vec<String>) {
    (Vec::new(), Vec::new())
}

/// Disabled compatibility entry point. Does not inspect the user's home.
pub fn resolve_known_marketplaces() -> Vec<ResolvedMarketplace> {
    Vec::new()
}

/// Disabled compatibility entry point. Does not inspect `claude_dir`.
pub fn resolve_known_marketplaces_in(_claude_dir: &Path) -> Vec<ResolvedMarketplace> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_enabled_disabled_fails_closed() {
        let json = serde_json::json!({
            "enabledPlugins": {
                "alpha@marketplace": true,
                "beta@marketplace": false,
                "gamma@other": true
            }
        });
        let (enabled, disabled) = parse_enabled_disabled_plugins(&json);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
    }

    #[test]
    fn parse_enabled_disabled_empty() {
        let json = serde_json::json!({});
        let (enabled, disabled) = parse_enabled_disabled_plugins(&json);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
    }

    #[test]
    fn parse_enabled_disabled_unqualified_names_fail_closed() {
        let json = serde_json::json!({
            "enabledPlugins": {
                "plain-name": true,
                "other-name": false
            }
        });
        let (enabled, disabled) = parse_enabled_disabled_plugins(&json);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
    }

    #[test]
    fn parse_enabled_disabled_mixed_values_fail_closed() {
        let json = serde_json::json!({
            "enabledPlugins": {
                "good@m": true,
                "bad@m": "yes",
                "ugly@m": 42
            }
        });
        let (enabled, disabled) = parse_enabled_disabled_plugins(&json);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
    }

    #[test]
    fn filesystem_entry_points_fail_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{"enabledPlugins": {"foo@m": true, "bar@m": false}}"#,
        )
        .unwrap();

        let (enabled, disabled) = load_enabled_disabled_plugins(&path);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
        assert!(resolve(tmp.path()).is_empty());
        assert!(resolve_known_marketplaces().is_empty());
        assert!(resolve_known_marketplaces_in(tmp.path()).is_empty());
    }

    #[test]
    fn parse_enabled_disabled_conflict_fails_closed() {
        let json = serde_json::json!({
            "enabledPlugins": {
                "conflict@market1": true,
                "conflict@market2": false
            }
        });
        let (enabled, disabled) = parse_enabled_disabled_plugins(&json);
        assert!(enabled.is_empty());
        assert!(disabled.is_empty());
    }
}
