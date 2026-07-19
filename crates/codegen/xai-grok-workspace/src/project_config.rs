//! Project config-file discovery: locating repo-local `.mcp.json` and
//! `.grok/config.toml` files by walking from `cwd` up to the git root.
//!
//! These pure `git2` + filesystem walks are shared by the shell's config
//! loaders and the folder-trust gate's `repo_configs_present`.

use std::path::{Path, PathBuf};

use xai_grok_agent::repo::RepoDirChain;

/// Filename of the project-local MCP server config.
pub const MCP_JSON_FILENAME: &str = ".mcp.json";

/// Candidate `.mcp.json` paths from repo root to `cwd`, whether or not they exist.
/// Useful for file watching so newly created files are detected after startup.
pub fn mcp_json_candidate_paths(cwd: &Path) -> Vec<PathBuf> {
    let Some(cwd) = xai_grok_config::validate_grok_path(cwd) else {
        return Vec::new();
    };
    mcp_json_candidate_paths_in(&RepoDirChain::resolve(&cwd).dirs)
}

/// [`mcp_json_candidate_paths`] over a precomputed cwd→git-root dir chain
/// ([`RepoDirChain`]), repo-root-first. Private: only this file's two callers.
fn mcp_json_candidate_paths_in(chain_dirs: &[PathBuf]) -> Vec<PathBuf> {
    chain_dirs
        .iter()
        .rev()
        .filter_map(|dir| {
            xai_grok_config::validate_grok_path(dir)?;
            xai_grok_config::validate_grok_path(&dir.join(MCP_JSON_FILENAME))
        })
        .collect()
}

/// Find existing `.mcp.json` files from `cwd` up to the git root (repo-root-first order).
pub fn find_mcp_json_files(cwd: &Path) -> Vec<PathBuf> {
    let Some(cwd) = xai_grok_config::validate_grok_path(cwd) else {
        return Vec::new();
    };
    find_mcp_json_files_in(&RepoDirChain::resolve(&cwd).dirs)
}

/// [`find_mcp_json_files`] over a precomputed dir chain. See [`RepoDirChain`].
/// `pub(crate)` — the gate (`repo_configs_present`) reaches it within this crate.
pub(crate) fn find_mcp_json_files_in(chain_dirs: &[PathBuf]) -> Vec<PathBuf> {
    mcp_json_candidate_paths_in(chain_dirs)
        .into_iter()
        .filter(|path| xai_grok_config::validate_grok_path(path).is_some() && path.is_file())
        .collect()
}

/// True when `config_path` is `$GROK_HOME/config.toml` (user tier, not project).
fn is_user_grok_config_file(config_path: &Path) -> bool {
    let Some(user_home) = xai_grok_config::user_grok_home() else {
        return false;
    };
    let user_config = user_home.join("config.toml");
    if config_path == user_config.as_path() {
        return true;
    }
    let Ok(canonical_config) = dunce::canonicalize(config_path) else {
        return false;
    };
    let canonical_user = dunce::canonicalize(&user_config).unwrap_or(user_config);
    canonical_config == canonical_user
}

/// Find all `.grok/config.toml` files from `cwd` upward to the git repo root.
/// Returns paths ordered from repo root (lowest priority) to cwd (highest priority),
/// matching the convention used by skills and AGENTS.md discovery.
///
/// If no git repo is found, only checks `cwd/.grok/config.toml`. Excludes the
/// user-global config so `cwd == $HOME` does not treat `~/.grok/config.toml` as
/// a project overlay.
pub fn find_project_configs(cwd: &Path) -> Vec<PathBuf> {
    let Some(cwd) = xai_grok_config::validate_grok_path(cwd) else {
        return Vec::new();
    };
    find_project_configs_in(&RepoDirChain::resolve(&cwd).dirs)
}

/// [`find_project_configs`] over a precomputed cwd→git-root dir chain
/// ([`RepoDirChain`]), repo-root-first. Excludes the user-global config so
/// `cwd == $HOME` does not treat `~/.grok/config.toml` as a project overlay.
/// `pub(crate)` — the gate (`repo_configs_present`) reaches it within this crate.
pub(crate) fn find_project_configs_in(chain_dirs: &[PathBuf]) -> Vec<PathBuf> {
    // `dirs` is cwd-first; reverse so repo root comes first (lowest priority)
    // and cwd last (highest), matching skills/AGENTS.md discovery order.
    chain_dirs
        .iter()
        .rev()
        .filter_map(|dir| {
            xai_grok_config::validate_grok_path(dir)?;
            xai_grok_config::validate_grok_path(&dir.join(".grok").join("config.toml"))
        })
        .filter(|config_path| {
            xai_grok_config::validate_grok_path(config_path).is_some()
                && config_path.is_file()
                && !is_user_grok_config_file(config_path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_project_configs_excludes_user_grok_config_file() {
        let Some(user_home) = xai_grok_config::user_grok_home() else {
            return;
        };
        let user_config = user_home.join("config.toml");
        if user_config.is_file() {
            #[allow(deprecated)]
            let home = std::env::home_dir().expect("home dir");
            let from_home = find_project_configs(&home);
            assert!(
                !from_home.iter().any(|p| is_user_grok_config_file(p)),
                "user config leaked into project configs: {from_home:?}"
            );
            assert!(is_user_grok_config_file(&user_config));
        }

        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("repo");
        std::fs::create_dir_all(project.join(".grok")).unwrap();
        std::fs::write(project.join(".grok/config.toml"), "# project\n").unwrap();
        let found = find_project_configs(&project);
        assert_eq!(found.len(), 1);
        assert!(!is_user_grok_config_file(&found[0]));
    }

    #[cfg(unix)]
    #[test]
    fn find_project_configs_rejects_symlink_into_vendor_state() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("repo");
        let vendor_config = project.join(".codex").join("config.toml");
        std::fs::create_dir_all(project.join(".grok")).unwrap();
        std::fs::create_dir_all(vendor_config.parent().unwrap()).unwrap();
        std::fs::write(&vendor_config, "# vendor\n").unwrap();
        symlink(&vendor_config, project.join(".grok").join("config.toml")).unwrap();

        assert!(find_project_configs(&project).is_empty());
    }

    #[test]
    fn project_config_discovery_rejects_vendor_cwds_and_keeps_near_misses() {
        let tmp = tempfile::tempdir().unwrap();
        let vendor = tmp.path().join(".codex").join("repo");
        let ordinary = tmp.path().join("codex-tools");

        for dir in [&vendor, &ordinary] {
            std::fs::create_dir_all(dir.join(".grok")).unwrap();
            std::fs::write(dir.join(MCP_JSON_FILENAME), "{}").unwrap();
            std::fs::write(dir.join(".grok/config.toml"), "# project\n").unwrap();
        }

        assert!(mcp_json_candidate_paths(&vendor).is_empty());
        assert!(find_mcp_json_files(&vendor).is_empty());
        assert!(find_project_configs(&vendor).is_empty());

        assert_eq!(
            mcp_json_candidate_paths(&ordinary),
            vec![ordinary.join(MCP_JSON_FILENAME)]
        );
        assert_eq!(
            find_mcp_json_files(&ordinary),
            vec![ordinary.join(MCP_JSON_FILENAME)]
        );
        assert_eq!(
            find_project_configs(&ordinary),
            vec![ordinary.join(".grok/config.toml")]
        );
    }

    #[test]
    fn precomputed_chain_filters_vendor_candidates_before_existence_checks() {
        let tmp = tempfile::tempdir().unwrap();
        let vendor = tmp.path().join(".claude").join("repo");
        let ordinary = tmp.path().join("claude-tools");
        std::fs::create_dir_all(&vendor).unwrap();
        std::fs::create_dir_all(&ordinary).unwrap();
        std::fs::write(vendor.join(MCP_JSON_FILENAME), "vendor").unwrap();
        std::fs::write(ordinary.join(MCP_JSON_FILENAME), "ordinary").unwrap();

        let chain = vec![vendor, ordinary.clone()];
        assert_eq!(
            mcp_json_candidate_paths_in(&chain),
            vec![ordinary.join(MCP_JSON_FILENAME)]
        );
        assert_eq!(
            find_mcp_json_files_in(&chain),
            vec![ordinary.join(MCP_JSON_FILENAME)]
        );
    }
}
