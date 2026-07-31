//! Disabled legacy tool-alias lookup API.
//!
//! These accessors remain exported so downstream crates do not fail to compile,
//! but Claude Code tool names are intentionally not resolved to Grok tools.

use super::tool::ToolKind;

/// Legacy allowlist lookup. Claude Code tool aliases are not supported.
pub fn kind_for(_name: &str) -> Option<ToolKind> {
    None
}

/// Legacy matcher lookup. Claude Code matcher aliases are not supported.
pub fn grok_names_for(_name: &str) -> impl Iterator<Item = &'static str> {
    std::iter::empty()
}

/// Legacy reverse matcher lookup. Claude Code matcher aliases are not supported.
pub fn claude_names_for(_grok_name: &str) -> impl Iterator<Item = &'static str> {
    std::iter::empty()
}

/// Legacy registry iterator. The disabled registry contains no Grok names.
pub fn grok_names() -> impl Iterator<Item = &'static str> {
    std::iter::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_alias_lookups_fail_closed() {
        assert_eq!(kind_for("Read"), None);
        assert_eq!(kind_for("Bash"), None);
        assert_eq!(grok_names_for("Read").next(), None);
        assert_eq!(claude_names_for("read_file").next(), None);
        assert_eq!(grok_names().next(), None);
    }
}
