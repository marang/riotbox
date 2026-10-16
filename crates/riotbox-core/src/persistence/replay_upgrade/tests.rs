use std::{fs, io};

use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, UndoPolicy,
    },
    ids::ActionId,
    replay::build_committed_replay_plan,
    session::{ActionCommitRecord, ReplayPolicy, SessionFile},
    transport::CommitBoundaryState,
};

use super::{
    ReplayChronologyCopyDisposition, ReplayChronologyCopyError, copy_session_with_committed_order,
    copy_with_publication,
};

fn legacy() -> SessionFile {
    let mut session = SessionFile::new("generated-copy", "test", "2026-10-16");
    session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    for id in 1..=2 {
        session.action_log.actions.push(Action {
            id: ActionId(id),
            actor: ActorType::User,
            command: ActionCommand::TransportPlay,
            params: ActionParams::Empty,
            target: ActionTarget::default(),
            requested_at: 90 + id,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(100 + id),
            result: Some(ActionResult {
                accepted: true,
                summary: "generated".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "transport".into(),
            },
            explanation: None,
        });
        session.action_log.commit_records.push(ActionCommitRecord {
            action_id: ActionId(id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: 0,
                bar_index: 1,
                phrase_index: 1,
                scene_id: None,
            },
            commit_sequence: id as u32,
            committed_at: 100 + id,
            mc202_source_phrase_plan: None,
        });
    }
    session
}

fn bytes(session: &SessionFile) -> Vec<u8> {
    serde_json::to_vec_pretty(session).unwrap()
}

fn assert_refused_unchanged(bytes: &[u8]) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    fs::write(&input, bytes).unwrap();
    let error = copy_session_with_committed_order(&input, &output).unwrap_err();
    assert!(error.to_string().contains("no Session copy was published"));
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert!(!output.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn complete_legacy_copy_changes_only_policy_and_reloads_same_replay() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let original = legacy();
    let original_bytes = bytes(&original);
    fs::write(&input, &original_bytes).unwrap();
    let result = copy_session_with_committed_order(&input, &output).unwrap();
    assert_eq!(
        result.disposition,
        ReplayChronologyCopyDisposition::Upgraded
    );
    assert_eq!(result.previous_policy, ReplayPolicy::DeterministicPreferred);
    assert_eq!(
        result.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    assert_eq!((result.action_count, result.commit_record_count), (2, 2));
    assert_eq!(fs::read(&input).unwrap(), original_bytes);
    let output_bytes = fs::read(&output).unwrap();
    let expected = String::from_utf8(original_bytes).unwrap().replace(
        "\"DeterministicPreferred\"",
        "\"DeterministicCommittedOrderV2\"",
    );
    assert_eq!(output_bytes, expected.as_bytes());
    let decoded: SessionFile = serde_json::from_slice(&output_bytes).unwrap();
    let old_plan = build_committed_replay_plan(&original.action_log).unwrap();
    let new_plan = build_committed_replay_plan(&decoded.action_log).unwrap();
    assert_eq!(old_plan, new_plan);
    let mut expected = original;
    expected.action_log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    assert_eq!(decoded, expected);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn escaped_token_and_unknown_extensions_are_byte_exact_not_value_roundtripped() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let original = String::from_utf8(bytes(&legacy()))
        .unwrap()
        .replacen("\"action_log\"", "\"action_\\u006cog\"", 1)
        .replacen("\"replay_policy\"", "\"replay_\\u0070olicy\"", 1)
        .replace(
            "\"DeterministicPreferred\"",
            "\"Deterministic\\u0050referred\"",
        );
    let original = original.replacen('{', "{\n\"opaque\":{\"duplicate\":1,\"duplicate\":2,\"huge\":1e9999,\"integer\":184467440737095516160,\"negzero\":-0,\"spelling\":1.2300e+04,\"policy\":\"DeterministicPreferred\"},", 1)
        .replace("\"notes\": null", "\"notes\": \"literal \\\"replay_policy\\\" and \\u00e4\"");
    fs::write(&input, &original).unwrap();
    copy_session_with_committed_order(&input, &output).unwrap();
    let expected = original.replace(
        "\"Deterministic\\u0050referred\"",
        "\"DeterministicCommittedOrderV2\"",
    );
    assert_eq!(fs::read(&output).unwrap(), expected.as_bytes());
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
}

#[test]
fn already_v2_is_validated_then_copied_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let mut session = legacy();
    session.action_log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    let original = bytes(&session);
    fs::write(&input, &original).unwrap();
    let result = copy_session_with_committed_order(&input, &output).unwrap();
    assert_eq!(
        result.disposition,
        ReplayChronologyCopyDisposition::AlreadyCommittedOrder
    );
    assert_eq!(fs::read(&output).unwrap(), original);
    assert_eq!(fs::read(&input).unwrap(), original);
    session.action_log.commit_records.clear();
    assert_refused_unchanged(&bytes(&session));
}

#[test]
fn incomplete_permuted_ambiguous_snapshot_and_duplicate_histories_refuse() {
    let mut incomplete = legacy();
    incomplete.action_log.commit_records.pop();
    assert_refused_unchanged(&bytes(&incomplete));
    let mut permuted = legacy();
    permuted.action_log.commit_records.reverse();
    assert_refused_unchanged(&bytes(&permuted));
    let mut ambiguous = legacy();
    ambiguous.action_log.commit_records[0].boundary.beat_index = 12;
    assert_refused_unchanged(&bytes(&ambiguous));
    let mut snapshot = legacy();
    snapshot.action_log.replay_policy = ReplayPolicy::SnapshotOnly;
    assert_refused_unchanged(&bytes(&snapshot));
    let mut duplicate = legacy();
    duplicate.action_log.actions[1].id = ActionId(1);
    assert_refused_unchanged(&bytes(&duplicate));
}

#[test]
fn missing_duplicate_escaped_duplicate_unknown_and_malformed_known_fields_refuse() {
    let json = String::from_utf8(bytes(&legacy())).unwrap();
    let valid_log = serde_json::to_string(&legacy().action_log).unwrap();
    for changed in [
        json.replace(
            "\"replay_policy\": \"DeterministicPreferred\"",
            "\"other\": null",
        ),
        json.replace(
            "\"replay_policy\":",
            "\"replay_policy\":\"DeterministicPreferred\",\"replay_policy\":",
        ),
        json.replace(
            "\"replay_policy\":",
            "\"replay_\\u0070olicy\":\"DeterministicPreferred\",\"replay_policy\":",
        ),
        json.replace(
            "\"action_log\":",
            &format!("\"action_\\u006cog\":{valid_log},\"action_log\":"),
        ),
        json.replace(
            "\"session_id\":",
            "\"session_id\":\"duplicate\",\"session_id\":",
        ),
        json.replace("DeterministicPreferred", "UnknownPolicy"),
        json.replace("\"V1\"", "\"V999\""),
        json.replace("\"DeterministicPreferred\"", "null"),
        "{".into(),
    ] {
        assert_refused_unchanged(changed.as_bytes());
    }
}

#[test]
fn same_path_occupied_directory_different_or_missing_parent_refuse() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let original = bytes(&legacy());
    fs::write(&input, &original).unwrap();
    let occupied = dir.path().join("occupied.json");
    fs::write(&occupied, b"keep").unwrap();
    let other = dir.path().join("other");
    fs::create_dir(&other).unwrap();
    for output in [
        input.clone(),
        occupied.clone(),
        other.clone(),
        other.join("new.json"),
        dir.path().join("missing/new.json"),
        dir.path().join("other/."),
        dir.path().join("other/.."),
    ] {
        assert!(copy_session_with_committed_order(&input, &output).is_err());
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&occupied).unwrap(), b"keep");
    }
    assert!(!dir.path().join("missing").exists());
    assert!(copy_session_with_committed_order(&other, dir.path().join("out.json")).is_err());
    assert!(!dir.path().join("out.json").exists());
}

#[test]
fn lexical_reserved_storage_refuses_without_creating_output() {
    let dir = tempfile::tempdir().unwrap();
    let reserved = dir.path().join("graph.json.riotbox-graphs");
    fs::create_dir(&reserved).unwrap();
    let input = reserved.join("input.json");
    let original = bytes(&legacy());
    fs::write(&input, &original).unwrap();
    assert!(copy_session_with_committed_order(&input, reserved.join("new.json")).is_err());
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read_dir(&reserved).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn symlink_hardlink_and_resolved_reserved_paths_refuse_but_parent_alias_is_safe() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let original = bytes(&legacy());
    fs::write(&input, &original).unwrap();
    let link = dir.path().join("input-link.json");
    symlink(&input, &link).unwrap();
    let dangling = dir.path().join("dangling.json");
    symlink("absent", &dangling).unwrap();
    let hard = dir.path().join("hardlink.json");
    fs::hard_link(&input, &hard).unwrap();
    for output in [&link, &dangling, &hard] {
        assert!(copy_session_with_committed_order(&input, output).is_err());
    }
    assert!(copy_session_with_committed_order(&link, dir.path().join("copy.json")).is_err());
    assert_eq!(fs::read(&input).unwrap(), original);
    assert!(
        fs::symlink_metadata(&dangling)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let alias = dir.path().join("parent-alias");
    symlink(dir.path(), &alias).unwrap();
    copy_session_with_committed_order(&input, alias.join("copy.json")).unwrap();
    assert!(dir.path().join("copy.json").is_file());
    let reserved = dir.path().join("graph.json.riotbox-graphs");
    fs::create_dir(&reserved).unwrap();
    fs::write(reserved.join("input.json"), &original).unwrap();
    let hidden = dir.path().join("innocent-alias");
    symlink(&reserved, &hidden).unwrap();
    assert!(
        copy_session_with_committed_order(hidden.join("input.json"), hidden.join("new.json"))
            .is_err()
    );
    assert_eq!(fs::read_dir(&reserved).unwrap().count(), 1);
}

#[test]
fn publication_errors_preserve_input_and_keep_original_io_meaning() {
    for (kind, link_stage) in [
        (io::ErrorKind::Unsupported, true),
        (io::ErrorKind::Unsupported, false),
        (io::ErrorKind::PermissionDenied, true),
        (io::ErrorKind::WriteZero, false),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.json");
        let output = dir.path().join("output.json");
        let original = bytes(&legacy());
        fs::write(&input, &original).unwrap();
        let error = copy_with_publication(&input, &output, |_, _| {
            let source = io::Error::new(kind, "injected publication failure");
            Err(if link_stage {
                super::exclusive_file::ExclusiveFileError::Link(source)
            } else {
                super::exclusive_file::ExclusiveFileError::Preparation(source)
            })
        })
        .unwrap_err();
        match error {
            ReplayChronologyCopyError::PublicationUnsupported { source, .. } => {
                assert!(link_stage);
                assert_eq!(source.kind(), io::ErrorKind::Unsupported)
            }
            ReplayChronologyCopyError::Io { source, .. } => {
                assert!(!link_stage || kind != io::ErrorKind::Unsupported);
                assert_eq!(source.kind(), kind);
            }
            other => panic!("unexpected {other}"),
        }
        assert_eq!(fs::read(&input).unwrap(), original);
        assert!(!output.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}

#[test]
fn destination_creation_race_does_not_overwrite_winner_or_input() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let original = bytes(&legacy());
    fs::write(&input, &original).unwrap();
    let error = copy_with_publication(&input, &output, |path, bytes| {
        fs::write(path, b"race winner")
            .map_err(super::exclusive_file::ExclusiveFileError::Preparation)?;
        super::exclusive_file::publish(path, bytes)
    })
    .unwrap_err();
    assert!(
        matches!(error, ReplayChronologyCopyError::Io { source, .. } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&output).unwrap(), b"race winner");
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn relative_graph_capture_and_source_paths_keep_the_same_base_without_hydration() {
    use crate::{
        session::{CaptureRef, CaptureType, GraphStorageMode, SourceGraphRef, SourceRef},
        source_graph::{DecodeProfile, GraphProvenance, SourceGraphVersion},
    };
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let mut session = legacy();
    session.source_refs.push(SourceRef {
        source_id: "generated-source".into(),
        path_hint: "missing/source.wav".into(),
        content_hash: "metadata-only".into(),
        duration_seconds: 1.0,
        decode_profile: DecodeProfile::Native,
    });
    session.source_graph_refs.push(SourceGraphRef {
        source_id: "generated-source".into(),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "metadata-only".into(),
        storage_mode: GraphStorageMode::External,
        embedded_graph: None,
        external_path: Some("missing/graph.json".into()),
        provenance: GraphProvenance {
            sidecar_version: "test".into(),
            provider_set: vec![],
            generated_at: "test".into(),
            source_hash: "metadata-only".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    });
    session.captures.push(CaptureRef {
        capture_id: "generated-capture".into(),
        capture_type: CaptureType::Loop,
        audio_identity: None,
        source_origin_refs: vec![],
        source_window: None,
        lineage_capture_refs: vec![],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "missing/capture.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    });
    let original = bytes(&session);
    fs::write(&input, &original).unwrap();
    copy_session_with_committed_order(&input, &output).unwrap();
    let loaded = crate::persistence::load_session_json(&output).unwrap();
    assert_eq!(loaded.source_refs, session.source_refs);
    assert_eq!(loaded.source_graph_refs, session.source_graph_refs);
    assert_eq!(loaded.captures, session.captures);
    assert_eq!(output.parent(), input.parent());
    assert_eq!(fs::read(&input).unwrap(), original);
    // Missing referenced directory is never created, enumerated or hydrated.
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn absent_defaulted_fields_and_unknown_log_extensions_are_not_materialized() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    let json = String::from_utf8(bytes(&legacy())).unwrap()
        .replace("  \"export_receipts\": [],\n", "")
        .replace("\"action_log\": {", "\"action_log\": {\"opaque\":{\"replay_policy\":\"DeterministicPreferred\",\"x\":-0.000e+99},");
    assert!(!json.contains("export_receipts"));
    fs::write(&input, &json).unwrap();
    copy_session_with_committed_order(&input, &output).unwrap();
    let expected = json.replace(
        "\"replay_policy\": \"DeterministicPreferred\"",
        "\"replay_policy\": \"DeterministicCommittedOrderV2\"",
    );
    assert_eq!(fs::read(&output).unwrap(), expected.as_bytes());
    assert_eq!(fs::read(&input).unwrap(), json.as_bytes());
}
