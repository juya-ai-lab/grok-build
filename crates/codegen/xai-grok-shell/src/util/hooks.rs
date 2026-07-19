//! Shared hook source path discovery.

use std::path::{Path, PathBuf};

use xai_grok_hooks::discovery::HookSource;

/// Owned paths for hook sources. Callers borrow via `as_sources()`.
pub struct HookSourcePaths {
    pub global: Vec<PathBuf>,
    pub project: Vec<PathBuf>,
}

impl HookSourcePaths {
    /// Borrow as `HookSource` refs. Project sources are excluded when untrusted.
    pub fn as_sources(&self, include_project: bool) -> (Vec<HookSource<'_>>, Vec<HookSource<'_>>) {
        let global = self
            .global
            .iter()
            .filter_map(|p| allowed_path_to_source(p))
            .collect();
        let project = if include_project {
            self.project
                .iter()
                .filter_map(|p| allowed_path_to_source(p))
                .collect()
        } else {
            vec![]
        };
        (global, project)
    }
}

fn path_to_source(p: &Path) -> HookSource<'_> {
    if p.is_dir() {
        HookSource::Directory(p)
    } else {
        HookSource::SettingsFile(p)
    }
}

fn allowed_path_to_source(p: &Path) -> Option<HookSource<'_>> {
    xai_grok_config::validate_grok_path(p)?;
    Some(path_to_source(p))
}

/// Add a hook source only after the central lexical/canonical vendor-state
/// guard approves it. This must precede `path_to_source()`'s `is_dir()` probe.
fn push_allowed_hook_source(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if xai_grok_config::validate_grok_path(&path).is_some() {
        paths.push(path);
    } else {
        tracing::warn!(
            path = %path.display(),
            "refusing hook source under Claude/Codex vendor state"
        );
    }
}

/// Build hook source paths for global (`~/`) and project (`<git_root>/`) scopes.
/// Callers gate project sources on trust via `as_sources(trusted)`.
pub fn discover_hook_source_paths(
    git_root: Option<&Path>,
    compat: &xai_grok_tools::types::compat::CompatConfig,
) -> HookSourcePaths {
    // The build-wide kill switch is authoritative even if a caller manually
    // constructs a resolved config with the public field set to true.
    let skip_claude_compat = !xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED || !compat.claude.hooks;
    // Phase 2 cutoff: if the user has imported, skip .claude/settings.json
    // sources. Native .grok/hooks/ directories are still scanned (they hold
    // any hooks that were imported by /import-claude).
    let skip_claude = skip_claude_compat
        || crate::claude_import::is_claude_import_marked_with_log("discover_hook_source_paths");

    // Compat gate: skip Cursor hook sources when disabled.
    let skip_cursor = !xai_grok_config::CURSOR_COMPAT_ENABLED || !compat.cursor.hooks;

    let home = dirs::home_dir();
    // user_grok_home() is None when no home resolves, so inspect lists the same
    // sources a live session loads, instead of a cwd-relative .grok.
    let grok = xai_grok_config::user_grok_home();
    let mut global = Vec::new();

    if !skip_claude && let Some(ref h) = home {
        push_allowed_hook_source(&mut global, h.join(".claude").join("settings.json"));
        push_allowed_hook_source(&mut global, h.join(".claude").join("settings.local.json"));
    }
    if let Some(ref grok) = grok {
        push_allowed_hook_source(&mut global, grok.join("hooks"));
    }

    let custom_paths: Vec<PathBuf> = grok
        .as_ref()
        .and_then(|g| {
            let path = g.join("hooks-paths");
            if xai_grok_config::validate_grok_path(&path).is_none() {
                tracing::warn!(
                    path = %path.display(),
                    "refusing hook path registry under Claude/Codex vendor state"
                );
                return None;
            }
            std::fs::read_to_string(path).ok()
        })
        .map(|content| {
            content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| PathBuf::from(l.trim()))
                .filter(|path| {
                    let allowed = xai_grok_config::validate_grok_path(path).is_some();
                    if !allowed {
                        tracing::warn!(path = %path.display(), "refusing hook source under Claude/Codex vendor state");
                    }
                    allowed
                })
                .collect()
        })
        .unwrap_or_default();
    global.extend(custom_paths);

    if let Some(ref h) = home
        && !skip_cursor
    {
        push_allowed_hook_source(&mut global, h.join(".cursor").join("hooks.json"));
    }

    let mut project = Vec::new();

    if let Some(root) = git_root {
        if !skip_claude {
            push_allowed_hook_source(&mut project, root.join(".claude").join("settings.json"));
            push_allowed_hook_source(
                &mut project,
                root.join(".claude").join("settings.local.json"),
            );
        }
        push_allowed_hook_source(&mut project, root.join(".grok").join("hooks"));
        if !skip_cursor {
            push_allowed_hook_source(&mut project, root.join(".cursor").join("hooks.json"));
        }
    }

    HookSourcePaths { global, project }
}

/// Single load entry point: build compat-aware sources, gate project sources on
/// trust, then load. Every session-startup and mid-session reload site routes
/// through here so the source policy stays in one place. `discover_hook_source_paths`
/// and `HookSourcePaths::as_sources` stay public for the build-gated `inspect`
/// path and the unit tests that assert on the raw source lists.
pub fn discover_hooks(
    git_root: Option<&Path>,
    compat: &xai_grok_tools::types::compat::CompatConfig,
    trusted: bool,
) -> (
    xai_grok_hooks::discovery::HookRegistry,
    Vec<xai_grok_hooks::error::HookError>,
) {
    let source_paths = discover_hook_source_paths(git_root, compat);
    let (global_sources, project_sources) = source_paths.as_sources(trusted);
    xai_grok_hooks::discovery::load_hooks_from_sources(&global_sources, &project_sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_grok_hooks_directory_is_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks = tmp.path().join(".grok").join("hooks");
        std::fs::create_dir_all(&hooks).unwrap();

        let sources = discover_hook_source_paths(
            Some(tmp.path()),
            &xai_grok_tools::types::compat::CompatConfig::default(),
        );
        assert!(sources.project.contains(&hooks));
    }

    #[cfg(unix)]
    #[test]
    fn project_grok_hooks_symlink_into_vendor_state_is_rejected() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let vendor_hooks = tmp.path().join(".claude").join("hooks");
        std::fs::create_dir_all(tmp.path().join(".grok")).unwrap();
        std::fs::create_dir_all(&vendor_hooks).unwrap();
        symlink(&vendor_hooks, tmp.path().join(".grok").join("hooks")).unwrap();

        let sources = discover_hook_source_paths(
            Some(tmp.path()),
            &xai_grok_tools::types::compat::CompatConfig::default(),
        );
        assert!(
            !sources
                .project
                .contains(&tmp.path().join(".grok").join("hooks"))
        );
    }
}
