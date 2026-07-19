//! Build-disabled vendor compatibility end-to-end test.
//!
//! Each test builds a fake `$HOME` containing skills/rules/AGENTS.md under the
//! `.grok`, `.agents`, `.cursor`, and `.claude` dirs, then verifies that even
//! explicit runtime enablement cannot restore proprietary vendor sources while
//! native Grok and the original shared `.agents` skill/command sources remain
//! available.
//!
//! These are `#[ignore]` (they spawn a built binary) like the agent-type
//! invariant suite. Run locally:
//! ```bash
//! cargo test -p xai-grok-shell --test test_vendor_compat -- --ignored
//! ```

use std::future::Future;
use std::path::Path;

use xai_grok_test_support::*;

async fn with_local_set<F, Fut>(f: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    tokio::task::LocalSet::new().run_until(f()).await;
}

/// Unique markers placed in skill descriptions / file contents so assertions
/// can't be fooled by incidental occurrences of a bare word like "shell".
const MARKER_GROK_SKILL: &str = "ZZ_GROK_SKILL_MARKER";
const MARKER_AGENTS_SKILL: &str = "ZZ_AGENTS_SKILL_MARKER";
const MARKER_AGENTS_COMMAND: &str = "ZZ_AGENTS_COMMAND_MARKER";
const MARKER_CURSOR_SKILL: &str = "ZZ_CURSOR_SKILL_MARKER";
const MARKER_CLAUDE_SKILL: &str = "ZZ_CLAUDE_SKILL_MARKER";
const MARKER_CURSOR_RULE: &str = "ZZ_CURSOR_RULE_MARKER";
const MARKER_CLAUDE_RULE: &str = "ZZ_CLAUDE_RULE_MARKER";
const MARKER_CLAUDE_AGENTS: &str = "ZZ_CLAUDE_AGENTS_MARKER";
const MARKER_CURSOR_AGENTS: &str = "ZZ_CURSOR_AGENTS_MARKER";

fn write_file(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("path has parent")).expect("create dirs");
    std::fs::write(path, contents).expect("write file");
}

/// Write a `<vendor>/skills/<name>/SKILL.md` with the given description marker.
fn write_skill(home: &Path, vendor_dir: &str, name: &str, marker: &str) {
    let p = home
        .join(vendor_dir)
        .join("skills")
        .join(name)
        .join("SKILL.md");
    write_file(
        &p,
        &format!("---\nname: {name}\ndescription: {marker}\n---\n\nSkill body.\n"),
    );
}

/// Populate a fake `$HOME` + repo cwd with the full vendor-compat fixture set.
fn seed_fixtures(home: &Path, cwd: &Path) {
    // Skills (User scope, home-based).
    write_skill(home, ".grok", "grok-skill", MARKER_GROK_SKILL);
    write_skill(home, ".agents", "standard-skill", MARKER_AGENTS_SKILL);
    write_skill(home, ".cursor", "my-cursor-skill", MARKER_CURSOR_SKILL);
    write_skill(home, ".claude", "my-claude-skill", MARKER_CLAUDE_SKILL);
    write_file(
        &home
            .join(".agents")
            .join("commands")
            .join("standard-command.md"),
        &format!(
            "---\nname: standard-command\ndescription: {MARKER_AGENTS_COMMAND}\n---\n\nCommand body.\n"
        ),
    );

    // Rules: repo-local `.cursor/rules/r.md` and `.claude/rules/c.md`
    // (discovered via the cwd→root walk, gated by their respective rules cell).
    write_file(
        &cwd.join(".cursor").join("rules").join("r.md"),
        &format!("# rule\n{MARKER_CURSOR_RULE}\n"),
    );
    write_file(
        &cwd.join(".claude").join("rules").join("c.md"),
        &format!("# rule\n{MARKER_CLAUDE_RULE}\n"),
    );
    // AGENTS.md: `~/.claude/CLAUDE.md` and `~/.cursor/AGENTS.md`
    // (discovered via the home compat scan, gated by their respective agents cell).
    write_file(
        &home.join(".claude").join("CLAUDE.md"),
        &format!("# claude instructions\n{MARKER_CLAUDE_AGENTS}\n"),
    );
    write_file(
        &home.join(".cursor").join("AGENTS.md"),
        &format!("# cursor instructions\n{MARKER_CURSOR_AGENTS}\n"),
    );
}

/// Spawn the agent with the given compat env overrides, send one prompt, and
/// return every inference request body concatenated into one string for
/// substring assertions (system prompt + skill listing + injected reminders).
async fn run_scenario(env: &[(&str, &str)]) -> String {
    let server = MockInferenceServer::start()
        .await
        .expect("start mock server");
    let workdir = git_workdir();
    let home = tempfile::TempDir::new().expect("create temp home");
    seed_fixtures(home.path(), workdir.path());

    let client = GrokStdioClient::spawn_with_home_and_env(&server, workdir.path(), home, env).await;
    client.initialize_with_timeout().await;
    let session_id = client.create_session_with_timeout(workdir.path()).await;
    let _ = client.prompt_with_timeout(&session_id, "hello").await;

    let bodies: Vec<String> = server
        .requests()
        .iter()
        .filter_map(|e| e.body.as_ref().map(|b| b.to_string()))
        .collect();
    assert!(
        !bodies.is_empty(),
        "expected at least one inference request; stderr:\n{}",
        client.stderr()
    );
    bodies.join("\n---\n")
}

/// Explicit true values from every supported runtime source stay ineffective.
#[tokio::test]
#[ignore] // requires pre-built binary
async fn proprietary_vendor_compatibility_cannot_be_enabled() {
    with_local_set(|| async {
        let body = run_scenario(&[
            ("GROK_CURSOR_SKILLS_ENABLED", "true"),
            ("GROK_CURSOR_RULES_ENABLED", "true"),
            ("GROK_CURSOR_AGENTS_ENABLED", "true"),
            ("GROK_CLAUDE_SKILLS_ENABLED", "true"),
            ("GROK_CLAUDE_RULES_ENABLED", "true"),
            ("GROK_CLAUDE_AGENTS_ENABLED", "true"),
        ])
        .await;
        assert!(body.contains(MARKER_GROK_SKILL));
        assert!(body.contains(MARKER_AGENTS_SKILL));
        assert!(body.contains(MARKER_AGENTS_COMMAND));
        assert!(!body.contains(MARKER_CURSOR_SKILL));
        assert!(!body.contains(MARKER_CURSOR_RULE));
        assert!(!body.contains(MARKER_CURSOR_AGENTS));
        assert!(!body.contains(MARKER_CLAUDE_SKILL));
        assert!(!body.contains(MARKER_CLAUDE_RULE));
        assert!(!body.contains(MARKER_CLAUDE_AGENTS));
    })
    .await;
}
