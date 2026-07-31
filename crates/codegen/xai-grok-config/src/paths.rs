//! Filesystem locations for grok config files and binaries.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "macos")]
const CLAUDE_MANAGED_SETTINGS_PATH: &str =
    "/Library/Application Support/ClaudeCode/managed-settings.json";
#[cfg(target_os = "linux")]
const CLAUDE_MANAGED_SETTINGS_PATH: &str = "/etc/claude-code/managed-settings.json";

/// The default user grok directory (`~/.grok`, canonicalized) used when
/// `GROK_HOME` is unset. Exposed so callers (e.g. display helpers) can detect
/// whether [`grok_home()`] is the default without duplicating the computation.
///
/// Uses [`dunce::canonicalize`] instead of [`std::fs::canonicalize`]: on
/// Windows, std returns a verbatim path (`\\?\C:\Users\...`) which external
/// tools choke on — e.g. `git clone` rejects `\\?\` destinations with
/// "Invalid argument", breaking marketplace cache clones under
/// `~/.grok/marketplace-cache`. `dunce` strips the prefix whenever the path
/// is safely representable in legacy form; on non-Windows it is identical to
/// `std::fs::canonicalize`.
pub fn default_grok_home() -> PathBuf {
    #[allow(deprecated)]
    let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home).unwrap_or(home).join(".grok")
}

/// Resolve `$GROK_HOME` (or the normal `~/.grok` default) only when it is not
/// third-party agent-owned state.
///
/// This reads the live environment on every call so processes and test
/// harnesses that intentionally change `GROK_HOME` do not bypass validation via
/// [`GROK_HOME`]'s cache. An empty `GROK_HOME`, an unavailable user home, or a
/// path equal to/below the effective `CODEX_HOME`, `CLAUDE_CONFIG_DIR`,
/// `~/.claude`, `~/.cursor`, or the shared `~/.agents` root returns `None`, as
/// does a
/// path whose exact basename is `.claude.json`.
pub fn validated_grok_home() -> Option<PathBuf> {
    #[allow(deprecated)]
    let user_home = std::env::home_dir();
    let grok_home = match std::env::var_os("GROK_HOME") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        Some(_) => return None,
        None => {
            user_home.as_ref()?;
            default_grok_home()
        }
    };
    validate_grok_path_with(
        &grok_home,
        user_home.as_deref(),
        std::env::var_os("CODEX_HOME"),
        std::env::var_os("CLAUDE_CONFIG_DIR"),
    )
}

/// Return `path` unchanged only when it is outside vendor-specific state roots.
///
/// Existing paths (and the deepest existing ancestor of a not-yet-created
/// path) are canonicalized before the component-safe containment check. This
/// catches symlink aliases without using substring checks that would reject an
/// unrelated directory merely because its name contains `codex` or `claude`.
/// The exact `.claude.json` basename is also treated as vendor state.
pub fn validate_grok_path(path: &Path) -> Option<PathBuf> {
    #[allow(deprecated)]
    let user_home = std::env::home_dir();
    validate_grok_path_with(
        path,
        user_home.as_deref(),
        std::env::var_os("CODEX_HOME"),
        std::env::var_os("CLAUDE_CONFIG_DIR"),
    )
}

/// Perform the non-I/O portion of [`validate_grok_path`].
///
/// Rejects literal third-party agent state components and an exact
/// `.claude.json` path component. This is useful at mutation boundaries that
/// must fail before any canonicalization or filesystem metadata access.
/// Existing symlink aliases still require the full [`validate_grok_path`] check
/// afterwards.
pub fn validate_grok_path_lexically(path: &Path) -> Option<PathBuf> {
    if path.as_os_str().is_empty()
        || has_vendor_state_component(path)
        || has_vendor_state_basename(path)
    {
        return None;
    }
    Some(path.to_path_buf())
}

/// Return a skill or command source path only when it is outside proprietary
/// vendor state. Inside the shared `.agents` namespace, the original Grok
/// surfaces `.agents/skills` and `.agents/commands` are accepted.
///
/// The `.agents` directory itself is accepted as a discovery container, but
/// unsupported sibling surfaces such as `.agents/rules` and
/// `.agents/personas` remain outside Grok's boundary.
pub fn validate_skill_path(path: &Path) -> Option<PathBuf> {
    #[allow(deprecated)]
    let user_home = std::env::home_dir();
    validate_skill_path_with(
        path,
        user_home.as_deref(),
        std::env::var_os("CODEX_HOME"),
        std::env::var_os("CLAUDE_CONFIG_DIR"),
    )
}

/// Perform the non-I/O portion of [`validate_skill_path`].
pub fn validate_skill_path_lexically(path: &Path) -> Option<PathBuf> {
    if path.as_os_str().is_empty()
        || has_proprietary_vendor_state_component(path)
        || has_vendor_state_basename(path)
        || !agents_components_are_skill_or_command_scoped(path)
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn validate_grok_path_with(
    path: &Path,
    user_home: Option<&Path>,
    codex_home_env: Option<std::ffi::OsString>,
    claude_home_env: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    validate_grok_path_lexically(path)?;

    let candidate = path_for_comparison(path)?;
    validate_grok_path_lexically(&candidate)?;
    let codex_home = codex_home_env
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| user_home.map(|home| home.join(".codex")));
    let claude_home = claude_home_env
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let default_claude_home = user_home.map(|home| home.join(".claude"));
    let agents_home = user_home.map(|home| home.join(".agents"));

    for root in [codex_home, claude_home, default_claude_home, agents_home]
        .into_iter()
        .flatten()
    {
        let Some(root) = path_for_comparison(&root) else {
            // A root supplied by the environment that cannot even be made
            // absolute is not safe to reason around.
            return None;
        };
        if path_is_within(&candidate, &root) {
            return None;
        }
    }
    Some(path.to_path_buf())
}

fn validate_skill_path_with(
    path: &Path,
    user_home: Option<&Path>,
    codex_home_env: Option<std::ffi::OsString>,
    claude_home_env: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    validate_skill_path_lexically(path)?;

    let candidate = path_for_comparison(path)?;
    validate_skill_path_lexically(&candidate)?;
    let codex_home = codex_home_env
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| user_home.map(|home| home.join(".codex")));
    let claude_home = claude_home_env
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let default_claude_home = user_home.map(|home| home.join(".claude"));

    for root in [codex_home, claude_home, default_claude_home]
        .into_iter()
        .flatten()
    {
        let root = path_for_comparison(&root)?;
        if path_is_within(&candidate, &root) {
            return None;
        }
    }
    Some(path.to_path_buf())
}

fn has_vendor_state_component(path: &Path) -> bool {
    path.components().any(|component| {
        let component = component.as_os_str();
        [".claude", ".cursor", ".codex", ".agents", ".claude.json"]
            .into_iter()
            .any(|blocked| os_component_eq(component, std::ffi::OsStr::new(blocked)))
    })
}

fn has_proprietary_vendor_state_component(path: &Path) -> bool {
    path.components().any(|component| {
        let component = component.as_os_str();
        [".claude", ".cursor", ".codex", ".claude.json"]
            .into_iter()
            .any(|blocked| os_component_eq(component, std::ffi::OsStr::new(blocked)))
    })
}

fn agents_components_are_skill_or_command_scoped(path: &Path) -> bool {
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        if os_component_eq(component.as_os_str(), std::ffi::OsStr::new(".agents"))
            && components.peek().is_some_and(|next| {
                !os_component_eq(next.as_os_str(), std::ffi::OsStr::new("skills"))
                    && !os_component_eq(next.as_os_str(), std::ffi::OsStr::new("commands"))
            })
        {
            return false;
        }
    }
    true
}

fn has_vendor_state_basename(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| os_component_eq(name, std::ffi::OsStr::new(".claude.json")))
}

#[cfg(not(windows))]
fn os_component_eq(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    left == right
}

#[cfg(windows)]
fn os_component_eq(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

/// Produce an absolute path suitable for containment comparisons. If the full
/// path does not exist, canonicalize its deepest existing ancestor and append
/// the unresolved tail before lexical normalization. This preserves symlink
/// resolution for the part the filesystem can currently prove.
fn path_for_comparison(path: &Path) -> Option<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };

    if let Ok(canonical) = dunce::canonicalize(&absolute) {
        return Some(normalize_lexically(&canonical));
    }

    let mut ancestor = absolute.as_path();
    loop {
        if let Ok(canonical) = dunce::canonicalize(ancestor) {
            let tail = absolute.strip_prefix(ancestor).ok()?;
            return Some(normalize_lexically(&canonical.join(tail)));
        }
        ancestor = ancestor.parent()?;
    }
}

fn normalize_lexically(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match components.last() {
                Some(Component::Normal(_)) => {
                    components.pop();
                }
                Some(Component::RootDir) => {}
                _ => components.push(component),
            },
            _ => components.push(component),
        }
    }
    components.into_iter().collect()
}

#[cfg(not(windows))]
fn path_is_within(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

#[cfg(windows)]
fn path_is_within(path: &Path, root: &Path) -> bool {
    let mut path_components = path.components();
    root.components().all(|root_component| {
        path_components.next().is_some_and(|path_component| {
            path_component
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&root_component.as_os_str().to_string_lossy())
        })
    })
}

/// Per-user config directory: `$GROK_HOME` or `~/.grok`. Created if needed.
pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            let grok_home = validated_grok_home().unwrap_or_else(|| {
                #[allow(deprecated)]
                let fallback = (std::env::var_os("GROK_HOME").is_none()
                    && std::env::home_dir().is_none())
                .then(default_grok_home)
                .and_then(|path| validate_grok_path(&path));
                fallback.unwrap_or_else(|| invalid_grok_home())
            });
            let _ = std::fs::create_dir_all(&grok_home);
            grok_home
        })
        .clone()
}

fn invalid_grok_home() -> ! {
    panic!(
        "refusing invalid GROK_HOME: path is empty, named .claude.json, or inside CODEX_HOME, CLAUDE_CONFIG_DIR, ~/.claude, ~/.cursor, or ~/.agents state"
    )
}

/// The user-global grok home, but only when one genuinely and safely resolves:
/// `Some` when a valid `$GROK_HOME` is set or a home directory is found, `None`
/// otherwise. Unlike [`grok_home()`], this never falls back to a cwd-relative
/// `.grok`, so callers that *scan* user-global grok resources (hooks,
/// marketplace sources, ...) don't mistake a project's `.grok` tree for the
/// user-global one when no home resolves.
pub fn user_grok_home() -> Option<PathBuf> {
    let home = validated_grok_home()?;
    let _ = std::fs::create_dir_all(&home);
    Some(home)
}

/// Canonical grok application path: `$GROK_HOME/bin/grok` (Unix) or `grok.exe` (Windows).
pub fn grok_application() -> PathBuf {
    grok_application_in(&grok_home())
}

/// [`grok_application`] under an explicit home instead of `$GROK_HOME`.
pub fn grok_application_in(home: &std::path::Path) -> PathBuf {
    let name = if cfg!(windows) { "grok.exe" } else { "grok" };
    home.join("bin").join(name)
}

/// System-wide config directory: `/etc/grok/` on Unix, `None` on Windows.
pub fn system_config_dir() -> Option<PathBuf> {
    if cfg!(unix) {
        Some(PathBuf::from("/etc/grok"))
    } else {
        None
    }
}

/// System path for the managed-settings.json used for settings compat, if it exists.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn claude_managed_settings_path() -> Option<PathBuf> {
    if !crate::CLAUDE_CODE_COMPAT_ENABLED {
        return None;
    }
    let path = PathBuf::from(CLAUDE_MANAGED_SETTINGS_PATH);
    path.exists().then_some(path)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn claude_managed_settings_path() -> Option<PathBuf> {
    None
}

/// The platform path where managed-settings.json would live for settings
/// compat, whether or not it exists. `None` on unsupported platforms.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn claude_managed_settings_probe_path() -> Option<PathBuf> {
    crate::CLAUDE_CODE_COMPAT_ENABLED.then(|| PathBuf::from(CLAUDE_MANAGED_SETTINGS_PATH))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn claude_managed_settings_probe_path() -> Option<PathBuf> {
    None
}

/// Max bytes for a single directory name component (macOS APFS, Linux ext4,
/// NTFS all enforce 255 bytes).
const MAX_DIRNAME_BYTES: usize = 255;

/// Encode a CWD string into a filesystem-safe directory name component.
///
/// Short CWDs (URL-encoded form <= 255 bytes) use URL-encoding for backward
/// compatibility and human readability on disk.
///
/// Long CWDs (> 255 bytes encoded) use a compact `{slug}-{blake3_hex16}`
/// form that is always <= 57 bytes. Callers must write a `.cwd` metadata
/// file via [`ensure_sessions_cwd_dir`] so the original CWD can be
/// recovered by [`decode_cwd_from_dirname`].
pub fn encode_cwd_dirname(cwd: &str) -> String {
    let url_encoded = urlencoding::encode(cwd);
    if url_encoded.len() <= MAX_DIRNAME_BYTES {
        return url_encoded.into_owned();
    }
    let hash = blake3::hash(cwd.as_bytes());
    let hash16 = &hash.to_hex()[..16];
    let leaf = std::path::Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    let slug = slugify(leaf, 40);
    let slug = if slug.is_empty() { "workspace" } else { &slug };
    format!("{slug}-{hash16}")
}

/// Recover the original CWD from a sessions CWD directory.
///
/// Tries URL-decoding the directory name first (works for short/legacy dirs).
/// Falls back to reading a `.cwd` metadata file inside the directory (written
/// by [`ensure_sessions_cwd_dir`] for hash-based dirs).
pub fn decode_cwd_from_dirname(dir: &std::path::Path) -> Option<String> {
    let name = dir.file_name()?.to_str()?;
    if let Ok(decoded) = urlencoding::decode(name) {
        let s = decoded.into_owned();
        // URL-decoded absolute CWDs always start with `/` (Unix) or a drive
        // letter (Windows).  The slug-hash form never does, so this
        // distinguishes the two encodings unambiguously.
        if s.starts_with('/') || (cfg!(windows) && s.chars().nth(1) == Some(':')) {
            return Some(s);
        }
    }
    std::fs::read_to_string(dir.join(".cwd"))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Build the CWD-level session directory path:
/// `grok_home()/sessions/{encode_cwd_dirname(cwd)}`.
///
/// Does **not** create the directory on disk — use [`ensure_sessions_cwd_dir`]
/// when the directory must exist.
pub fn sessions_cwd_dir(cwd: &str) -> PathBuf {
    grok_home().join("sessions").join(encode_cwd_dirname(cwd))
}

/// Create the CWD-level session directory and write a `.cwd` metadata file
/// when hash-based encoding is used (long paths).
///
/// For short paths the `.cwd` file is not written because the directory name
/// itself is reversible via URL-decoding.
pub fn ensure_sessions_cwd_dir(cwd: &str) -> std::io::Result<PathBuf> {
    let encoded_name = encode_cwd_dirname(cwd);
    let dir = grok_home().join("sessions").join(&encoded_name);
    std::fs::create_dir_all(&dir)?;
    // Hash-based encoding is in use when the dirname differs from the
    // plain URL-encoded form.  Write a `.cwd` file so decode can recover
    // the original path.  O_CREAT|O_EXCL via create_new avoids TOCTOU
    // races with parallel session starts.
    if encoded_name != urlencoding::encode(cwd).as_ref() {
        let cwd_file = dir.join(".cwd");
        match std::fs::File::create_new(&cwd_file) {
            Ok(mut f) => {
                std::io::Write::write_all(&mut f, cwd.as_bytes())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Ok(dir)
}

/// Generate a URL-safe slug from a string.
///
/// Lowercases, replaces non-alphanumeric chars with `-`, collapses
/// consecutive dashes, and truncates to `max_len` characters.
fn slugify(input: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_dash = false;
    for c in input.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            prev_dash = false;
        } else if !prev_dash {
            result.push('-');
            prev_dash = true;
        }
    }
    let trimmed = result.trim_matches('-');
    trimmed.chars().take(max_len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn vendor_state_roots_and_subdirectories_are_rejected() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let codex_home = user_home.join(".codex");
        let claude_home = user_home.join(".claude");
        std::fs::create_dir_all(&codex_home).unwrap();
        std::fs::create_dir_all(&claude_home).unwrap();

        assert_eq!(
            validate_grok_path_with(&codex_home, Some(&user_home), None, None),
            None
        );
        assert_eq!(
            validate_grok_path_with(
                &codex_home.join("grok-state/not-created-yet"),
                Some(&user_home),
                None,
                None,
            ),
            None
        );
        assert_eq!(
            validate_grok_path_with(
                &claude_home.join("nested/grok-state"),
                Some(&user_home),
                None,
                None,
            ),
            None
        );
    }

    #[test]
    fn effective_codex_home_env_is_rejected() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let codex_home = tmp.path().join("custom-codex-state");
        std::fs::create_dir_all(&codex_home).unwrap();

        assert_eq!(
            validate_grok_path_with(
                &codex_home.join("grok"),
                Some(&user_home),
                Some(codex_home.into_os_string()),
                None,
            ),
            None
        );
    }

    #[test]
    fn claude_config_env_and_default_claude_roots_are_rejected() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let custom_claude = tmp.path().join("custom-claude-state");
        let default_claude = user_home.join(".claude");
        for root in [&custom_claude, &default_claude] {
            std::fs::create_dir_all(root).unwrap();
        }
        let claude_env = Some(custom_claude.clone().into_os_string());

        for path in [custom_claude.join("grok"), default_claude.join("grok")] {
            assert_eq!(
                validate_grok_path_with(&path, Some(&user_home), None, claude_env.clone(),),
                None
            );
        }
    }

    #[test]
    fn component_similar_normal_paths_remain_valid() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let normal = user_home.join("my-codex-cache").join("grok");

        assert_eq!(
            validate_grok_path_with(&normal, Some(&user_home), None, None),
            Some(normal)
        );

        let agents_md = user_home.join("project/AGENTS.md");
        assert_eq!(
            validate_grok_path_with(&agents_md, Some(&user_home), None, None),
            Some(agents_md)
        );
    }

    #[test]
    fn claude_json_basename_is_rejected_without_near_miss_false_positives() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(project.join("nested")).unwrap();

        for path in [
            project.join(".claude.json"),
            project.join(".claude.json/hooks"),
            PathBuf::from(".claude.json"),
            project.join("nested/../.claude.json"),
        ] {
            assert_eq!(
                validate_grok_path_with(&path, Some(&user_home), None, None),
                None,
                "{}",
                path.display()
            );
            assert_eq!(validate_grok_path_lexically(&path), None);
        }

        for path in [
            project.join(".claude.json.bak"),
            project.join("my.claude.json"),
        ] {
            assert_eq!(
                validate_grok_path_with(&path, Some(&user_home), None, None),
                Some(path.clone())
            );
            assert_eq!(validate_grok_path_lexically(&path), Some(path));
        }
    }

    #[cfg(windows)]
    #[test]
    fn claude_json_basename_is_case_insensitive_on_windows() {
        assert_eq!(
            validate_grok_path_lexically(Path::new(r"C:\project\.CLAUDE.JSON")),
            None
        );
    }

    #[test]
    fn vendor_named_components_are_rejected_anywhere_in_a_project() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let project = tmp.path().join("project");

        for component in [".claude", ".cursor", ".codex", ".agents"] {
            assert_eq!(
                validate_grok_path_with(
                    &project.join(component).join("grok-state"),
                    Some(&user_home),
                    None,
                    None,
                ),
                None
            );
        }
        let similarly_named = project.join(".codex-cache/grok-state");
        assert_eq!(
            validate_grok_path_with(&similarly_named, Some(&user_home), None, None),
            Some(similarly_named)
        );
        let standard_agents_skills = project.join(".agents/skills/example/SKILL.md");
        assert_eq!(
            validate_grok_path_with(&standard_agents_skills, Some(&user_home), None, None),
            None
        );
        assert_eq!(
            validate_skill_path_with(&standard_agents_skills, Some(&user_home), None, None,),
            Some(standard_agents_skills)
        );
    }

    #[test]
    fn skill_paths_accept_the_original_agents_skill_and_command_surfaces() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let project = tmp.path().join("project");

        for path in [
            project.join(".grok/skills/example/SKILL.md"),
            project.join(".agents"),
            project.join(".agents/skills/example/SKILL.md"),
            project.join(".agents/commands/example.md"),
            user_home.join(".agents/skills/example/SKILL.md"),
            user_home.join(".agents/commands/example.md"),
        ] {
            assert_eq!(
                validate_skill_path_with(&path, Some(&user_home), None, None),
                Some(path)
            );
        }

        for path in [
            project.join(".agents/rules/example.md"),
            project.join(".agents/personas/example.md"),
            project.join(".codex/skills/example/SKILL.md"),
        ] {
            assert_eq!(
                validate_skill_path_with(&path, Some(&user_home), None, None),
                None
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "refusing invalid GROK_HOME: path is empty, named .claude.json, or inside CODEX_HOME, CLAUDE_CONFIG_DIR, ~/.claude, ~/.cursor, or ~/.agents state"
    )]
    fn cached_grok_home_refuses_invalid_resolution_with_clear_error() {
        invalid_grok_home();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_alias_into_vendor_state_is_rejected() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let codex_home = user_home.join(".codex");
        let alias = tmp.path().join("apparently-grok");
        std::fs::create_dir_all(&codex_home).unwrap();
        std::os::unix::fs::symlink(&codex_home, &alias).unwrap();

        assert_eq!(
            validate_grok_path_with(
                &alias.join("nested/not-created-yet"),
                Some(&user_home),
                None,
                None,
            ),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn skill_validator_allows_agents_skills_and_commands_but_rejects_other_siblings() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let agents = user_home.join(".agents");
        let skills = agents.join("skills");
        let commands = agents.join("commands");
        let rules = agents.join("rules");
        let skills_alias = tmp.path().join("shared-skills");
        let commands_alias = tmp.path().join("shared-commands");
        let rules_alias = tmp.path().join("apparently-skills");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::create_dir_all(&commands).unwrap();
        std::fs::create_dir_all(&rules).unwrap();
        std::os::unix::fs::symlink(&skills, &skills_alias).unwrap();
        std::os::unix::fs::symlink(&commands, &commands_alias).unwrap();
        std::os::unix::fs::symlink(&rules, &rules_alias).unwrap();

        assert_eq!(
            validate_skill_path_with(&skills_alias, Some(&user_home), None, None),
            Some(skills_alias)
        );
        assert_eq!(
            validate_skill_path_with(&commands_alias, Some(&user_home), None, None),
            Some(commands_alias)
        );
        assert_eq!(
            validate_skill_path_with(&rules_alias, Some(&user_home), None, None),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_alias_to_claude_json_is_rejected_after_canonicalization() {
        let tmp = TempDir::new().unwrap();
        let user_home = tmp.path().join("home");
        let claude_json = tmp.path().join(".claude.json");
        let alias = tmp.path().join("apparently-safe.json");
        std::fs::write(&claude_json, "{}").unwrap();
        std::os::unix::fs::symlink(&claude_json, &alias).unwrap();

        assert_eq!(
            validate_grok_path_lexically(&alias),
            Some(alias.clone()),
            "the lexical phase cannot see through a safe-looking symlink"
        );
        assert_eq!(
            validate_grok_path_with(&alias, Some(&user_home), None, None),
            None
        );
    }

    /// Realistic CWDs that trigger the bug (URL-encoded > 255 bytes).
    const LONG_CWDS: &[&str] = &[
        "/Users/dev/Documents/開発プロジェクト/機能追加/テスト環境/ソースコード/main-branch",
        "/Users/user/Library/Mobile Documents/com~apple~CloudDocs/项目文件/深层嵌套目录/更深层次的/工作区域/project",
        "/Users/user/Library/CloudStorage/OneDrive-대한민국회사/프로젝트/개발환경/소스코드/백엔드/서비스/my-app",
        "/Users/user/Documents/工作文件夹/二零二六年项目/子目录一/子目录二/子目录三/源代码/code",
    ];

    #[test]
    fn long_cwd_uses_hash_fallback_within_name_max() {
        let long_cwd = format!("/Users/test/{}", "中".repeat(30));
        let encoded = encode_cwd_dirname(&long_cwd);
        assert!(encoded.len() <= MAX_DIRNAME_BYTES);
        assert!(!encoded.starts_with("%2F"));
    }

    #[test]
    fn different_long_paths_produce_different_hashes() {
        let a = format!("/Users/test/{}", "中".repeat(30));
        let b = format!("/Users/test/{}", "日".repeat(30));
        assert_ne!(encode_cwd_dirname(&a), encode_cwd_dirname(&b));
    }

    #[test]
    fn decode_reads_cwd_file_for_hash_dirs() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("some-slug-abcdef0123456789");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".cwd"), "/original/long/path").unwrap();
        assert_eq!(
            decode_cwd_from_dirname(&dir),
            Some("/original/long/path".to_string())
        );
    }

    #[test]
    fn decode_returns_none_without_cwd_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("some-slug-abcdef0123456789");
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(decode_cwd_from_dirname(&dir), None);
    }

    #[test]
    fn cwd_file_write_is_idempotent_via_excl() {
        let tmp = TempDir::new().unwrap();
        let long_cwd = format!("/Users/test/{}", "中".repeat(30));
        let dir = tmp.path().join(encode_cwd_dirname(&long_cwd));
        std::fs::create_dir_all(&dir).unwrap();
        let cwd_file = dir.join(".cwd");
        std::fs::write(&cwd_file, &long_cwd).unwrap();
        match std::fs::File::create_new(&cwd_file) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            other => panic!("expected AlreadyExists, got: {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&cwd_file).unwrap(), long_cwd);
    }

    #[test]
    fn url_encoded_long_cwd_fails_on_real_filesystem() {
        let tmp = TempDir::new().unwrap();
        let url_encoded = urlencoding::encode(LONG_CWDS[0]).into_owned();
        let result = std::fs::create_dir_all(tmp.path().join(&url_encoded));
        assert!(result.is_err());
    }

    #[test]
    fn full_roundtrip_on_real_filesystem_for_long_cwds() {
        let tmp = TempDir::new().unwrap();
        for cwd in LONG_CWDS {
            let encoded = encode_cwd_dirname(cwd);
            let dir = tmp.path().join(&encoded);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(".cwd"), cwd).unwrap();
            assert_eq!(decode_cwd_from_dirname(&dir).as_deref(), Some(*cwd));
        }
    }

    #[test]
    fn short_cwds_use_url_encoding_and_roundtrip_on_real_filesystem() {
        let tmp = TempDir::new().unwrap();
        for cwd in [
            "/Users/foo/project",
            "/tmp",
            "/Users/user/Documents/project-名前",
        ] {
            let encoded = encode_cwd_dirname(cwd);
            assert_eq!(encoded, urlencoding::encode(cwd).into_owned());
            let dir = tmp.path().join(&encoded);
            std::fs::create_dir_all(&dir).unwrap();
            assert_eq!(decode_cwd_from_dirname(&dir).as_deref(), Some(cwd));
        }
    }

    #[test]
    fn default_grok_home_has_no_verbatim_prefix() {
        // On Windows, std::fs::canonicalize returns `\\?\C:\...` verbatim
        // paths that external tools (notably `git clone`) reject. The dunce
        // canonicalization must yield a plain path. No-op assertion on Unix.
        let home = default_grok_home();
        assert!(!home.to_string_lossy().starts_with(r"\\?\"));
        assert!(home.ends_with(".grok"));
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World!", 40), "hello-world");
    }

    #[test]
    fn slugify_cjk_produces_empty() {
        assert_eq!(slugify("深层目录", 40), "");
    }

    #[test]
    fn slugify_truncates() {
        assert_eq!(slugify(&"a".repeat(100), 10).len(), 10);
    }
}
