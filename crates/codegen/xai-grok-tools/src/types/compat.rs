//! Vendor compatibility configuration for third-party agent surfaces
//! (skills, rules, agents, MCPs, hooks, sessions).
//!
//! Historically the agent hard-coded the dir lists `[".grok", ".agents",
//! ".claude", ".cursor"]` (and `RULES_DIRS` / `AGENT_FILENAMES`) across ~6
//! call sites in three crates. This module now owns the canonical cell registry
//! used by runtime resolution and diagnostics (env var → config TOML → remote
//! setting → compiled default). Third-party compatibility is build-disabled
//! and cannot be re-enabled by runtime inputs.
//!
//! Two forms:
//! - [`CompatConfigToml`] — raw, parsed from the `[compat]` TOML section. Each
//!   cell is `Option<bool>` so `None` falls through to the resolution chain.
//! - [`CompatConfig`] — resolved plain bools consumed at runtime. Every
//!   third-party vendor cell is forced off in this build.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatVendor {
    Cursor,
    Claude,
    Codex,
}

impl CompatVendor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    /// Whether this vendor's compatibility layer is enabled in this build.
    pub const fn is_build_enabled(self) -> bool {
        match self {
            Self::Cursor => xai_grok_config::CURSOR_COMPAT_ENABLED,
            Self::Claude => xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED,
            Self::Codex => xai_grok_config::CODEX_COMPAT_ENABLED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatSurface {
    Skills,
    Rules,
    Agents,
    Mcps,
    Hooks,
    Sessions,
}

impl CompatSurface {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skills => "skills",
            Self::Rules => "rules",
            Self::Agents => "agents",
            Self::Mcps => "mcps",
            Self::Hooks => "hooks",
            Self::Sessions => "sessions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatRemoteKey {
    CursorSkills,
    CursorRules,
    CursorAgents,
    CursorMcps,
    CursorHooks,
    CursorSessions,
    ClaudeSkills,
    ClaudeRules,
    ClaudeAgents,
    ClaudeMcps,
    ClaudeHooks,
    ClaudeSessions,
    CodexSessions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatCell {
    vendor: CompatVendor,
    surface: CompatSurface,
    env_var: &'static str,
    remote_key: Option<CompatRemoteKey>,
}

impl CompatCell {
    const fn new(
        vendor: CompatVendor,
        surface: CompatSurface,
        env_var: &'static str,
        remote_key: Option<CompatRemoteKey>,
    ) -> Self {
        Self {
            vendor,
            surface,
            env_var,
            remote_key,
        }
    }

    pub const fn vendor(self) -> CompatVendor {
        self.vendor
    }

    pub const fn surface(self) -> CompatSurface {
        self.surface
    }

    pub const fn env_var(self) -> &'static str {
        self.env_var
    }

    pub const fn remote_key(self) -> Option<CompatRemoteKey> {
        self.remote_key
    }

    /// Whether Grok currently implements this compatibility surface.
    ///
    /// All third-party cells are build-disabled. The registry remains so the
    /// raw config shape and diagnostics stay stable without activating reads.
    pub const fn is_runtime_supported(self) -> bool {
        match self.vendor {
            CompatVendor::Cursor => self.vendor.is_build_enabled(),
            CompatVendor::Claude => self.vendor.is_build_enabled(),
            CompatVendor::Codex => {
                self.vendor.is_build_enabled() && matches!(self.surface, CompatSurface::Sessions)
            }
        }
    }
}

pub const COMPAT_CELLS: [CompatCell; 18] = [
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Skills,
        "GROK_CURSOR_SKILLS_ENABLED",
        Some(CompatRemoteKey::CursorSkills),
    ),
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Rules,
        "GROK_CURSOR_RULES_ENABLED",
        Some(CompatRemoteKey::CursorRules),
    ),
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Agents,
        "GROK_CURSOR_AGENTS_ENABLED",
        Some(CompatRemoteKey::CursorAgents),
    ),
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Mcps,
        "GROK_CURSOR_MCPS_ENABLED",
        Some(CompatRemoteKey::CursorMcps),
    ),
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Hooks,
        "GROK_CURSOR_HOOKS_ENABLED",
        Some(CompatRemoteKey::CursorHooks),
    ),
    CompatCell::new(
        CompatVendor::Cursor,
        CompatSurface::Sessions,
        "GROK_CURSOR_SESSIONS_ENABLED",
        Some(CompatRemoteKey::CursorSessions),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Skills,
        "GROK_CLAUDE_SKILLS_ENABLED",
        Some(CompatRemoteKey::ClaudeSkills),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Rules,
        "GROK_CLAUDE_RULES_ENABLED",
        Some(CompatRemoteKey::ClaudeRules),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Agents,
        "GROK_CLAUDE_AGENTS_ENABLED",
        Some(CompatRemoteKey::ClaudeAgents),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Mcps,
        "GROK_CLAUDE_MCPS_ENABLED",
        Some(CompatRemoteKey::ClaudeMcps),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Hooks,
        "GROK_CLAUDE_HOOKS_ENABLED",
        Some(CompatRemoteKey::ClaudeHooks),
    ),
    CompatCell::new(
        CompatVendor::Claude,
        CompatSurface::Sessions,
        "GROK_CLAUDE_SESSIONS_ENABLED",
        Some(CompatRemoteKey::ClaudeSessions),
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Skills,
        "GROK_CODEX_SKILLS_ENABLED",
        None,
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Rules,
        "GROK_CODEX_RULES_ENABLED",
        None,
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Agents,
        "GROK_CODEX_AGENTS_ENABLED",
        None,
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Mcps,
        "GROK_CODEX_MCPS_ENABLED",
        None,
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Hooks,
        "GROK_CODEX_HOOKS_ENABLED",
        None,
    ),
    CompatCell::new(
        CompatVendor::Codex,
        CompatSurface::Sessions,
        "GROK_CODEX_SESSIONS_ENABLED",
        Some(CompatRemoteKey::CodexSessions),
    ),
];

/// Raw per-vendor compat cells parsed from `[compat.<vendor>]` TOML.
///
/// For build-enabled vendors, resolution order is env override, this value,
/// remote flag, compiled default. Build-disabled vendors ignore these inputs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VendorCompatToml {
    pub skills: Option<bool>,
    pub rules: Option<bool>,
    pub agents: Option<bool>,
    pub mcps: Option<bool>,
    pub hooks: Option<bool>,
    pub sessions: Option<bool>,
}

impl VendorCompatToml {
    fn value(&self, surface: CompatSurface) -> Option<bool> {
        match surface {
            CompatSurface::Skills => self.skills,
            CompatSurface::Rules => self.rules,
            CompatSurface::Agents => self.agents,
            CompatSurface::Mcps => self.mcps,
            CompatSurface::Hooks => self.hooks,
            CompatSurface::Sessions => self.sessions,
        }
    }
}

/// Raw `[compat]` TOML section.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CompatConfigToml {
    #[serde(default)]
    pub cursor: VendorCompatToml,
    #[serde(default)]
    pub claude: VendorCompatToml,
    #[serde(default)]
    pub codex: VendorCompatToml,
}

impl CompatConfigToml {
    pub fn value(&self, cell: CompatCell) -> Option<bool> {
        match cell.vendor() {
            CompatVendor::Cursor => self.cursor.value(cell.surface()),
            CompatVendor::Claude => self.claude.value(cell.surface()),
            CompatVendor::Codex => self.codex.value(cell.surface()),
        }
    }
}

/// Resolved per-vendor compat cells. Plain bools — the runtime source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorCompat {
    pub skills: bool,
    pub rules: bool,
    pub agents: bool,
    pub mcps: bool,
    pub hooks: bool,
    pub sessions: bool,
}

impl VendorCompat {
    const fn with_all(value: bool) -> Self {
        Self {
            skills: value,
            rules: value,
            agents: value,
            mcps: value,
            hooks: value,
            sessions: value,
        }
    }

    fn value(&self, surface: CompatSurface) -> bool {
        match surface {
            CompatSurface::Skills => self.skills,
            CompatSurface::Rules => self.rules,
            CompatSurface::Agents => self.agents,
            CompatSurface::Mcps => self.mcps,
            CompatSurface::Hooks => self.hooks,
            CompatSurface::Sessions => self.sessions,
        }
    }

    fn set(&mut self, surface: CompatSurface, value: bool) {
        match surface {
            CompatSurface::Skills => self.skills = value,
            CompatSurface::Rules => self.rules = value,
            CompatSurface::Agents => self.agents = value,
            CompatSurface::Mcps => self.mcps = value,
            CompatSurface::Hooks => self.hooks = value,
            CompatSurface::Sessions => self.sessions = value,
        }
    }
}

impl Default for VendorCompat {
    fn default() -> Self {
        Self::with_all(true)
    }
}

/// Resolved `[compat]` configuration threaded into compatibility consumers.
///
/// All third-party cells are build-disabled. The vendor sections remain
/// schema-compatible, but their resolved values are always false.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatConfig {
    pub cursor: VendorCompat,
    pub claude: VendorCompat,
    pub codex: VendorCompat,
}

impl Default for CompatConfig {
    fn default() -> Self {
        Self {
            cursor: VendorCompat::with_all(xai_grok_config::CURSOR_COMPAT_ENABLED),
            claude: VendorCompat::with_all(xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED),
            codex: VendorCompat::with_all(xai_grok_config::CODEX_COMPAT_ENABLED),
        }
    }
}

impl CompatConfig {
    pub fn value(&self, cell: CompatCell) -> bool {
        match cell.vendor() {
            CompatVendor::Cursor => {
                xai_grok_config::CURSOR_COMPAT_ENABLED && self.cursor.value(cell.surface())
            }
            CompatVendor::Claude => {
                xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED && self.claude.value(cell.surface())
            }
            CompatVendor::Codex => {
                xai_grok_config::CODEX_COMPAT_ENABLED && self.codex.value(cell.surface())
            }
        }
    }

    pub fn set(&mut self, cell: CompatCell, value: bool) {
        match cell.vendor() {
            CompatVendor::Cursor => self.cursor.set(
                cell.surface(),
                value && xai_grok_config::CURSOR_COMPAT_ENABLED,
            ),
            CompatVendor::Claude => self.claude.set(
                cell.surface(),
                value && xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED,
            ),
            CompatVendor::Codex => self.codex.set(
                cell.surface(),
                value && xai_grok_config::CODEX_COMPAT_ENABLED,
            ),
        }
    }

    /// Config directories that may contain `skills/` subdirectories, in
    /// priority order. Native `.grok` and the shared Agent Skills standard
    /// root `.agents` are always included. Vendor-specific paths are excluded.
    pub fn skill_config_dirs(&self) -> Vec<&'static str> {
        let mut dirs = vec![".grok", ".agents"];
        if xai_grok_config::CURSOR_COMPAT_ENABLED && self.cursor.skills {
            dirs.push(".cursor");
        }
        dirs
    }

    /// Subdirectories scanned for `*.md` rules files. `.grok/rules` is always
    /// included; third-party vendor paths are never recognized.
    pub fn rules_dirs(&self) -> Vec<&'static str> {
        let mut dirs = vec![".grok/rules"];
        if xai_grok_config::CURSOR_COMPAT_ENABLED && self.cursor.rules {
            dirs.push(".cursor/rules");
        }
        dirs
    }

    /// Filenames recognized as project-instruction files. Claude-specific root
    /// names and `.claude/`-prefixed entries are never recognized.
    pub fn agent_filenames(&self) -> Vec<&'static str> {
        vec!["Agents.md", "AGENT.md", "AGENTS.md"]
    }

    /// Home-level vendor directories scanned for project instructions and
    /// rules. Third-party vendor paths are never recognized.
    pub fn agents_home_dirs(&self) -> Vec<&'static str> {
        let mut dirs = Vec::new();
        if xai_grok_config::CURSOR_COMPAT_ENABLED && self.cursor.agents {
            dirs.push(".cursor");
        }
        dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_and_defaults_cover_every_cell() {
        use CompatRemoteKey::*;

        assert_eq!(
            COMPAT_CELLS.map(|cell| {
                (
                    cell.vendor().as_str(),
                    cell.surface().as_str(),
                    cell.remote_key(),
                )
            }),
            [
                ("cursor", "skills", Some(CursorSkills)),
                ("cursor", "rules", Some(CursorRules)),
                ("cursor", "agents", Some(CursorAgents)),
                ("cursor", "mcps", Some(CursorMcps)),
                ("cursor", "hooks", Some(CursorHooks)),
                ("cursor", "sessions", Some(CursorSessions)),
                ("claude", "skills", Some(ClaudeSkills)),
                ("claude", "rules", Some(ClaudeRules)),
                ("claude", "agents", Some(ClaudeAgents)),
                ("claude", "mcps", Some(ClaudeMcps)),
                ("claude", "hooks", Some(ClaudeHooks)),
                ("claude", "sessions", Some(ClaudeSessions)),
                ("codex", "skills", None),
                ("codex", "rules", None),
                ("codex", "agents", None),
                ("codex", "mcps", None),
                ("codex", "hooks", None),
                ("codex", "sessions", Some(CodexSessions)),
            ]
        );

        assert!(!xai_grok_config::CURSOR_COMPAT_ENABLED);
        assert!(!xai_grok_config::CLAUDE_CODE_COMPAT_ENABLED);
        assert!(!xai_grok_config::CODEX_COMPAT_ENABLED);
        let defaults = CompatConfig::default();
        for cell in COMPAT_CELLS {
            assert!(
                !defaults.value(cell),
                "{}.{}",
                cell.vendor().as_str(),
                cell.surface().as_str()
            );
        }
        let vendor = defaults.cursor;
        assert_eq!(vendor, VendorCompat::with_all(false));
        assert_eq!(defaults.claude, VendorCompat::with_all(false));
        assert_eq!(defaults.codex, VendorCompat::with_all(false));

        assert_eq!(
            COMPAT_CELLS
                .into_iter()
                .filter(|cell| cell.is_runtime_supported())
                .map(|cell| (cell.vendor().as_str(), cell.surface().as_str()))
                .collect::<Vec<_>>(),
            Vec::<(&str, &str)>::new()
        );
    }

    #[test]
    fn build_disabled_cells_cannot_be_enabled_through_resolved_config() {
        let mut config = CompatConfig::default();
        for cell in COMPAT_CELLS {
            config.set(cell, true);
            assert!(
                !config.value(cell),
                "{}.{}",
                cell.vendor().as_str(),
                cell.surface().as_str()
            );
        }
        assert_eq!(config.claude, VendorCompat::with_all(false));
        assert_eq!(config.codex, VendorCompat::with_all(false));
        assert_eq!(config.cursor, VendorCompat::with_all(false));
    }

    #[test]
    fn skill_config_dirs_keep_native_and_shared_standard_roots() {
        assert_eq!(
            CompatConfig::default().skill_config_dirs(),
            vec![".grok", ".agents"]
        );
    }

    #[test]
    fn skill_config_dirs_gates_cursor() {
        let mut c = CompatConfig::default();
        c.cursor.skills = false;
        assert_eq!(c.skill_config_dirs(), vec![".grok", ".agents"]);
    }

    #[test]
    fn rules_dirs_exclude_all_vendor_roots() {
        assert_eq!(CompatConfig::default().rules_dirs(), vec![".grok/rules"]);
    }

    #[test]
    fn rules_dirs_gates_cursor() {
        let mut c = CompatConfig::default();
        c.cursor.rules = false;
        assert_eq!(c.rules_dirs(), vec![".grok/rules"]);
    }

    #[test]
    fn agent_filenames_exclude_all_claude_names() {
        assert_eq!(
            CompatConfig::default().agent_filenames(),
            vec!["Agents.md", "AGENT.md", "AGENTS.md"]
        );
    }

    #[test]
    fn claude_fields_cannot_restore_discovery_paths() {
        let mut c = CompatConfig::default();
        c.claude = VendorCompat::with_all(true);
        c.cursor = VendorCompat::with_all(true);
        assert_eq!(c.skill_config_dirs(), vec![".grok", ".agents"]);
        assert_eq!(c.rules_dirs(), vec![".grok/rules"]);
        assert_eq!(
            c.agent_filenames(),
            vec!["Agents.md", "AGENT.md", "AGENTS.md"]
        );
        assert!(c.agents_home_dirs().is_empty());
    }

    #[test]
    fn codex_fields_cannot_restore_shared_skill_roots() {
        let mut c = CompatConfig::default();
        c.codex = VendorCompat::with_all(true);
        c.cursor = VendorCompat::with_all(true);
        assert_eq!(c.skill_config_dirs(), vec![".grok", ".agents"]);
        assert_eq!(
            c.agent_filenames(),
            vec!["Agents.md", "AGENT.md", "AGENTS.md"]
        );
    }

    #[test]
    fn agents_home_dirs_gates_cursor() {
        let mut c = CompatConfig::default();
        assert!(c.agents_home_dirs().is_empty());
        c.cursor.agents = true;
        assert!(c.agents_home_dirs().is_empty());
    }

    #[test]
    fn toml_struct_deserializes_partial_cells() {
        // The raw TOML struct is parsed from `[compat]` in the shell crate
        // (where `toml` is a dep). Here we exercise the same serde shape via
        // YAML (available in this crate) to pin the `Option<bool>` + `#[serde(default)]`
        // semantics: unset cells stay `None`, unset vendors default-construct.
        let parsed: CompatConfigToml = serde_yaml::from_str(
            "cursor:\n  skills: false\n  sessions: true\ncodex:\n  sessions: true\n",
        )
        .unwrap();
        assert_eq!(parsed.cursor.skills, Some(false));
        assert_eq!(parsed.cursor.rules, None);
        assert_eq!(parsed.cursor.sessions, Some(true));
        assert_eq!(parsed.claude, VendorCompatToml::default());
        assert_eq!(parsed.codex.sessions, Some(true));
        assert_eq!(parsed.codex.skills, None);

        // mcps cell round-trips the same way.
        let parsed: CompatConfigToml = serde_yaml::from_str("claude:\n  mcps: false\n").unwrap();
        assert_eq!(parsed.claude.mcps, Some(false));
        assert_eq!(parsed.claude.hooks, None);
        assert_eq!(parsed.claude.sessions, None);
        assert_eq!(parsed.cursor, VendorCompatToml::default());
        assert_eq!(parsed.codex, VendorCompatToml::default());
    }
}
