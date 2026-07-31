use super::*;
use xai_grok_shell::session::unified_list::ListScope;

use crate::views::modal::ActiveModal;
use crate::views::session_picker::{PickerItem, SourceFilter, build_entry_map};
use xai_grok_workspace::foreign_sessions::ForeignSessionTool;

fn make_foreign_entry(
    id: &str,
    source: &str,
    cwd: &str,
) -> crate::app::app_view::SessionPickerEntry {
    let mut entry = make_picker_entry(id, cwd);
    entry.source = source.into();
    entry
}

fn at(
    mut entry: crate::app::app_view::SessionPickerEntry,
    seconds: i64,
) -> crate::app::app_view::SessionPickerEntry {
    let timestamp = chrono::DateTime::from_timestamp(seconds, 0).unwrap();
    entry.updated_at = timestamp;
    entry.last_active_at = Some(timestamp);
    entry
}

fn content_hit(id: &str) -> xai_grok_shell::extensions::session_search::SearchSessionHit {
    xai_grok_shell::extensions::session_search::SearchSessionHit {
        session_id: id.into(),
        summary: id.into(),
        cwd: "/repo".into(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        snippet: Some("native transcript match".into()),
        score: 1.0,
        matched_fields: vec![],
    }
}

fn modal_entries(app: &AppView) -> &[crate::app::app_view::SessionPickerEntry] {
    let Some(ActiveModal::SessionPicker {
        entries: Some(entries),
        ..
    }) = app.agents[&AgentId(0)].active_modal.as_ref()
    else {
        panic!("modal picker missing");
    };
    entries
}

#[test]
#[test]
fn foreign_result_drops_injected_disabled_vendor_rows() {
    let mut app = test_app();
    app.foreign_session_scan_seq = 5;
    app.session_picker_lanes.foreign_loading = true;
    app.session_picker_entries = Some(vec![make_picker_entry("native", "/repo")]);

    let _ = dispatch(
        Action::TaskComplete(TaskResult::ForeignSessionsScanned {
            entries: vec![
                make_foreign_entry("claude-injected", "claude", "/repo"),
                make_foreign_entry("codex-injected", "codex", "/repo"),
            ],
            seq: 5,
        }),
        &mut app,
    );

    let entries = app.session_picker_entries.as_ref().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "native");
}

#[test]
#[test]
#[test]
#[test]
#[test]
#[test]
fn foreign_empty_then_native_empty_finishes_once_without_resurrecting() {
    let mut app = test_app();
    app.session_picker_list_seq = 3;
    app.foreign_session_scan_seq = 5;
    app.session_picker_loading = true;
    app.session_picker_lanes.foreign_loading = true;

    let _ = dispatch(
        Action::TaskComplete(TaskResult::ForeignSessionsScanned {
            entries: vec![],
            seq: 5,
        }),
        &mut app,
    );
    assert!(!app.session_picker_lanes.foreign_loading);
    assert!(app.session_picker_loading);

    let _ = dispatch(
        Action::TaskComplete(TaskResult::SessionListLoaded {
            scope: ListScope::Cwd,
            sessions: vec![],
            partial: None,
            seq: 3,
            query: None,
        }),
        &mut app,
    );
    assert!(!app.session_picker_loading);
    assert!(app.session_picker_entries.is_none());
    assert!(app.session_picker_lanes.pending_notice.is_none());
}

#[test]
#[test]
fn modal_empty_notice_waits_until_both_lanes_are_empty() {
    let mut app = test_app_with_agent();
    app.session_picker_list_seq = 9;
    app.foreign_session_scan_seq = 10;
    open_session_picker_with(&mut app, vec![]);
    if let Some(ActiveModal::SessionPicker { loading, lanes, .. }) = get_active_agent_mut(&mut app)
        .unwrap()
        .active_modal
        .as_mut()
    {
        *loading = true;
        lanes.foreign_loading = true;
    }
    let _ = dispatch(
        Action::TaskComplete(TaskResult::SessionListLoaded {
            scope: ListScope::Cwd,
            sessions: vec![],
            partial: None,
            seq: 9,
            query: None,
        }),
        &mut app,
    );
    assert!(app.agents[&AgentId(0)].toast.is_none());

    let _ = dispatch(
        Action::TaskComplete(TaskResult::ForeignSessionsScanned {
            entries: vec![],
            seq: 10,
        }),
        &mut app,
    );
    assert!(read_toast(&app).contains("No sessions found"));
}

#[test]
#[test]
#[test]
fn external_filter_clears_and_suppresses_native_content_state() {
    let mut app = test_app();
    app.session_picker_grouped = false;
    app.session_picker_entries = Some(vec![
        make_picker_entry("native", "/repo"),
        make_foreign_entry("foreign", "codex", "/repo"),
    ]);
    app.session_picker_content_results = Some(vec![content_hit("native-hit")]);
    app.session_picker_content_loading = true;
    app.session_picker_state.expanded.insert(0);
    // Grok cycles straight into External.
    app.session_picker_source_filter = SourceFilter::Grok;
    let old_detail_generation = app.session_picker_detail_generation;

    let effects = dispatch(Action::CycleSessionSourceFilter, &mut app);

    assert!(effects.is_empty());
    assert_eq!(app.session_picker_source_filter, SourceFilter::External);
    assert!(app.session_picker_content_results.is_none());
    assert!(!app.session_picker_content_loading);
    assert!(app.session_picker_state.expanded.is_empty());
    assert!(app.session_picker_detail_generation > old_detail_generation);
    assert!(dispatch(Action::TriggerDeepSearch, &mut app).is_empty());
    assert!(
        dispatch(
            Action::ExpandSessionCard {
                source: "local".into(),
                session_id: "native".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(
        dispatch(
            Action::DeleteSession {
                source: "local".into(),
                session_id: "native".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(
        dispatch(
            Action::PickContentSession {
                session_id: "native-hit".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(
        dispatch(
            Action::PickContentSessionInWorktree {
                session_id: "native-hit".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .is_empty()
    );

    let map = build_entry_map(
        app.session_picker_entries.as_deref(),
        Some(&[content_hit("native-hit")]),
        "",
        false,
        true,
        SourceFilter::External,
        None,
    );
    assert_eq!(map.len(), 1);
    let Some(PickerItem::Fuzzy { original_index }) = map[0].as_ref() else {
        panic!("external row missing");
    };
    assert_eq!(
        app.session_picker_entries.as_ref().unwrap()[*original_index].id,
        "foreign"
    );
}

#[test]
fn modal_external_filter_clears_native_content_and_blocks_forced_search() {
    let mut app = test_app_with_agent();
    open_session_picker_with(
        &mut app,
        vec![
            make_picker_entry("native", "/repo"),
            make_foreign_entry("foreign", "claude", "/repo"),
        ],
    );
    if let Some(ActiveModal::SessionPicker {
        state,
        content_results,
        content_loading,
        source_filter,
        ..
    }) = get_active_agent_mut(&mut app)
        .unwrap()
        .active_modal
        .as_mut()
    {
        // Grok cycles straight into External.
        *source_filter = SourceFilter::Grok;
        *content_results = Some(vec![content_hit("native-hit")]);
        *content_loading = true;
        state.set_query("native");
        state.expanded.insert(0);
    }

    let _ = dispatch(Action::CycleSessionSourceFilter, &mut app);

    let Some(ActiveModal::SessionPicker {
        state,
        content_results,
        content_loading,
        source_filter,
        ..
    }) = app.agents[&AgentId(0)].active_modal.as_ref()
    else {
        panic!("modal picker missing");
    };
    assert_eq!(*source_filter, SourceFilter::External);
    assert!(content_results.is_none());
    assert!(!*content_loading);
    assert!(state.expanded.is_empty());
    assert!(dispatch(Action::ForceDeepSearch, &mut app).is_empty());
}

#[test]
fn cycle_reaches_every_filter_with_foreign_present() {
    // One press from the default reveals externals, and Local/Remote stay
    // reachable on the same plain cycle even with foreign rows loaded.
    let mut app = test_app();
    app.session_picker_entries = Some(vec![
        make_picker_entry("native", "/repo"),
        make_foreign_entry("foreign", "claude", "/repo"),
    ]);
    for expected in [
        SourceFilter::External,
        SourceFilter::All,
        SourceFilter::Local,
        SourceFilter::Remote,
        SourceFilter::Grok,
    ] {
        let _ = dispatch(Action::CycleSessionSourceFilter, &mut app);
        assert_eq!(app.session_picker_source_filter, expected);
    }
}

#[test]
fn active_modal_owns_stale_and_external_deep_search_results() {
    for external in [false, true] {
        let mut app = test_app_with_agent();
        open_session_picker_with(&mut app, vec![make_picker_entry("modal", "/repo")]);
        app.session_picker_deep_search_seq = 7;
        if let Some(ActiveModal::SessionPicker {
            deep_search_seq,
            source_filter,
            ..
        }) = get_active_agent_mut(&mut app)
            .unwrap()
            .active_modal
            .as_mut()
        {
            *deep_search_seq = if external { 7 } else { 8 };
            if external {
                *source_filter = SourceFilter::External;
            }
        }

        let _ = dispatch(
            Action::TaskComplete(TaskResult::DeepSearchResults {
                results: vec![content_hit("must-not-reach-welcome")],
                seq: 7,
            }),
            &mut app,
        );

        assert!(app.session_picker_content_results.is_none());
        let Some(ActiveModal::SessionPicker {
            content_results, ..
        }) = app.agents[&AgentId(0)].active_modal.as_ref()
        else {
            panic!("modal picker missing");
        };
        assert!(content_results.is_none());
    }
}

#[test]
fn detail_result_revalidates_source_id_and_generation_after_reorder() {
    let mut app = test_app_with_agent();
    open_session_picker_with(
        &mut app,
        vec![
            at(make_picker_entry("target", "/repo"), 20),
            at(make_picker_entry("other", "/repo"), 10),
        ],
    );
    if let Some(ActiveModal::SessionPicker { lanes, .. }) = get_active_agent_mut(&mut app)
        .unwrap()
        .active_modal
        .as_mut()
    {
        lanes.foreign_loading = true;
    }
    let effects = dispatch(
        Action::ExpandSessionCard {
            source: "local".into(),
            session_id: "target".into(),
        },
        &mut app,
    );
    let [
        Effect::LoadCardDetail {
            source,
            session_id,
            generation,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("expected identity-addressed detail effect");
    };
    assert_eq!(source, "local");
    assert_eq!(session_id, "target");
    let stale_generation = *generation;

    app.foreign_session_scan_seq = 4;
    let _ = dispatch(
        Action::TaskComplete(TaskResult::ForeignSessionsScanned {
            entries: vec![at(make_foreign_entry("new", "cursor", "/repo"), 30)],
            seq: 4,
        }),
        &mut app,
    );
    let detail = crate::app::app_view::CardDetail {
        turn_count: 7,
        tool_call_count: 3,
        first_prompt_preview: "first".into(),
    };
    let _ = dispatch(
        Action::TaskComplete(TaskResult::CardDetailLoaded {
            source: "local".into(),
            session_id: "target".into(),
            generation: stale_generation,
            detail: detail.clone(),
        }),
        &mut app,
    );
    assert!(
        modal_entries(&app)
            .iter()
            .all(|entry| entry.card_detail.is_none())
    );

    if let Some(ActiveModal::SessionPicker {
        entries: Some(entries),
        ..
    }) = get_active_agent_mut(&mut app)
        .unwrap()
        .active_modal
        .as_mut()
    {
        entries
            .iter_mut()
            .find(|entry| entry.id == "target")
            .unwrap()
            .source = "cursor".into();
    }
    let _ = dispatch(
        Action::TaskComplete(TaskResult::CardDetailLoaded {
            source: "local".into(),
            session_id: "target".into(),
            generation: app.session_picker_detail_generation,
            detail: detail.clone(),
        }),
        &mut app,
    );
    assert!(
        modal_entries(&app)
            .iter()
            .all(|entry| entry.card_detail.is_none())
    );
    if let Some(ActiveModal::SessionPicker {
        entries: Some(entries),
        ..
    }) = get_active_agent_mut(&mut app)
        .unwrap()
        .active_modal
        .as_mut()
    {
        entries
            .iter_mut()
            .find(|entry| entry.id == "target")
            .unwrap()
            .source = "local".into();
    }

    let _ = dispatch(
        Action::TaskComplete(TaskResult::CardDetailLoaded {
            source: "local".into(),
            session_id: "target".into(),
            generation: app.session_picker_detail_generation,
            detail,
        }),
        &mut app,
    );
    assert_eq!(
        modal_entries(&app)
            .iter()
            .find(|entry| entry.id == "target")
            .and_then(|entry| entry.card_detail.as_ref())
            .map(|detail| detail.turn_count),
        Some(7)
    );
}

#[test]
fn colliding_native_and_foreign_ids_use_source_at_initiation() {
    let mut app = test_app_with_agent();
    open_session_picker_with(
        &mut app,
        vec![
            make_foreign_entry("shared-id", "codex", "/repo"),
            make_picker_entry("shared-id", "/repo"),
        ],
    );

    let effects = dispatch(
        Action::ExpandSessionCard {
            source: "local".into(),
            session_id: "shared-id".into(),
        },
        &mut app,
    );
    assert!(matches!(
        effects.as_slice(),
        [Effect::LoadCardDetail {
            source,
            session_id,
            ..
        }] if source == "local" && session_id == "shared-id"
    ));
    assert!(
        dispatch(
            Action::ExpandSessionCard {
                source: "codex".into(),
                session_id: "shared-id".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(
        dispatch(
            Action::DeleteSession {
                source: "codex".into(),
                session_id: "shared-id".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(matches!(
        dispatch(
            Action::DeleteSession {
                source: "local".into(),
                session_id: "shared-id".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .as_slice(),
        [Effect::DeleteSession { session_id, .. }] if session_id == "shared-id"
    ));
}

#[test]
#[test]
fn disabled_vendor_picker_rows_and_deferred_resumes_are_rejected() {
    for (wire, tool) in [
        ("claude", ForeignSessionTool::Claude),
        ("codex", ForeignSessionTool::Codex),
    ] {
        let mut injected = test_app();
        injected.session_picker_entries = Some(vec![make_foreign_entry("blocked", wire, "/repo")]);
        assert!(dispatch(Action::PickSession(0), &mut injected).is_empty());
        assert!(injected.agents.is_empty());
        assert!(injected.deferred_startup.is_empty());

        let mut deferred = test_app();
        deferred.deferred_startup.session = Some(
            crate::app::session_startup::DeferredSessionStartup::ForeignResume {
                tool,
                native_id: "blocked".into(),
            },
        );
        assert!(drain_startup_actions(&mut deferred).is_empty());
        assert!(deferred.agents.is_empty());
        assert!(deferred.deferred_startup.is_empty());
    }
}

#[test]
#[test]
fn foreign_selection_and_mutation_guards_remain_central() {
    let mut app = test_app_with_agent();
    open_session_picker_with(
        &mut app,
        vec![make_foreign_entry("foreign-id", "cursor", "/repo")],
    );
    assert!(dispatch(Action::PickSessionInWorktree(0), &mut app).is_empty());
    assert!(
        dispatch(
            Action::ExpandSessionCard {
                source: "cursor".into(),
                session_id: "foreign-id".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(
        dispatch(
            Action::DeleteSession {
                source: "cursor".into(),
                session_id: "foreign-id".into(),
                cwd: "/repo".into(),
            },
            &mut app,
        )
        .is_empty()
    );
    assert!(app.agents[&AgentId(0)].active_modal.is_some());
}

#[test]
fn chat_picker_never_launches_or_accepts_foreign_scan() {
    let mut app = test_app();
    app.chat_mode = true;
    app.foreign_session_compat =
        xai_grok_workspace::foreign_sessions::EnabledForeignSessionSources {
            claude: true,
            codex: true,
            cursor: true,
        };
    let effects = dispatch(Action::FetchSessionList, &mut app);
    assert!(matches!(
        effects.as_slice(),
        [Effect::FetchSessionList { .. }]
    ));
    app.session_picker_entries = Some(vec![make_conversation_entry("chat")]);
    let _ = dispatch(
        Action::TaskComplete(TaskResult::ForeignSessionsScanned {
            entries: vec![make_foreign_entry("foreign", "claude", "/repo")],
            seq: app.foreign_session_scan_seq,
        }),
        &mut app,
    );
    assert_eq!(app.session_picker_entries.as_ref().unwrap().len(), 1);
}
