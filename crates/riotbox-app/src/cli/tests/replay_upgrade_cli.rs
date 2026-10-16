use crate::cli::args::parse_args;
use crate::cli::model::{AppLaunch, LaunchMode};
use crate::cli::replay_upgrade::run;
use riotbox_core::{
    persistence::{ReplayChronologyCopyDisposition, load_session_json, save_session_json},
    replay::build_committed_replay_plan,
    session::{ReplayPolicy, SessionFile, SourceRef},
    source_graph::DecodeProfile,
};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

const MODE: &str = "--replay-chronology-upgrade-copy";
const DESTINATION: &str = "--replay-chronology-destination";

fn arguments(input: &Path, destination: &Path) -> Vec<String> {
    vec![
        MODE.into(),
        "--session".into(),
        input.to_str().unwrap().into(),
        DESTINATION.into(),
        destination.to_str().unwrap().into(),
    ]
}

fn launch(input: &Path, destination: &Path) -> AppLaunch {
    parse_args(arguments(input, destination)).unwrap()
}

#[test]
fn replay_chronology_copy_is_reachable_with_both_explicit_paths() {
    let result = parse_args([
        "--replay-chronology-upgrade-copy".into(),
        "--session".into(),
        "original.json".into(),
        "--replay-chronology-destination".into(),
        "upgraded.json".into(),
    ]);
    let launch = result.expect("explicit chronology copy is reachable");
    assert!(launch.observer_path.is_none());
    match launch.mode {
        LaunchMode::ReplayChronologyUpgradeCopy {
            session_path,
            destination_path,
        } => {
            assert_eq!(session_path, Path::new("original.json"));
            assert_eq!(destination_path, Path::new("upgraded.json"));
        }
        other => panic!("expected explicit chronology copy, got {other:?}"),
    }
}

#[test]
fn copy_mode_requires_single_explicit_mode_and_paths_without_defaults() {
    let valid = arguments(Path::new("original.json"), Path::new("upgraded.json"));
    for positions in [&[0][..], &[1, 2][..], &[3, 4][..]] {
        let missing = valid
            .iter()
            .enumerate()
            .filter(|(index, _)| !positions.contains(index))
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        assert!(parse_args(missing).is_err());
    }
    for extra in [
        vec![MODE],
        vec!["--session", "other.json"],
        vec![DESTINATION, "other.json"],
    ] {
        let mut duplicate = valid.clone();
        duplicate.extend(extra.into_iter().map(str::to_owned));
        assert!(parse_args(duplicate).unwrap_err().contains("exactly once"));
    }
    for invalid in [
        vec![MODE, "--session"],
        vec![MODE, "--session", "", DESTINATION, "copy.json"],
        vec![MODE, "--session", DESTINATION, "copy.json"],
        vec![MODE, "--session", "original.json", DESTINATION],
    ] {
        assert!(parse_args(invalid.into_iter().map(str::to_owned)).is_err());
    }
}

#[test]
fn copy_mode_refuses_competing_options_in_either_argument_order() {
    let valid = arguments(Path::new("original.json"), Path::new("upgraded.json"));
    for extra in [
        vec!["--source", "never-read.wav"],
        vec!["--graph", "never-read.json"],
        vec!["--observer", "never-created.ndjson"],
        vec!["--sidecar", "never-executed.py"],
        vec!["--seed", "23"],
        vec!["--source-bpm", "120"],
        vec!["--live-recording-readiness-report"],
        vec!["--live-master-recording-execute"],
        vec!["--stem-package-local-ci-dry-run"],
        vec!["--daw-session-writer-plan"],
        vec!["--help"],
        vec!["unexpected.json"],
    ] {
        for prepend in [false, true] {
            let extra = extra
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>();
            let mixed = if prepend {
                [extra, valid.clone()].concat()
            } else {
                [valid.clone(), extra].concat()
            };
            assert!(parse_args(mixed).unwrap_err().contains("exclusive"));
        }
    }
}

#[test]
fn copy_help_explains_the_explicit_new_sibling_and_reference_limits() {
    let help = parse_args(["--help".into()]).unwrap_err();
    assert!(help.contains(MODE) && help.contains(DESTINATION));
    assert!(help.contains("Both paths are mandatory"));
    assert!(help.contains("without changing the original or verifying references"));
}

#[test]
fn headless_copy_reports_real_core_upgrade_without_hydrating_missing_references() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("original.json");
    let destination = dir.path().join("upgraded.json");
    let reference = dir.path().join("never-opened-generated-source.wav");
    let mut session = SessionFile::new("generated-cli-upgrade", "test", "2026-10-16");
    session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    session.source_refs.push(SourceRef {
        source_id: "generated-metadata-only".into(),
        path_hint: reference.to_str().unwrap().into(),
        content_hash: "generated-metadata-only".into(),
        duration_seconds: 32.0,
        decode_profile: DecodeProfile::Native,
    });
    save_session_json(&input, &session).unwrap();
    let before = fs::read(&input).unwrap();
    let launch = launch(&input, &destination);
    assert!(crate::cli::launch::load_state(launch.mode.clone()).is_err());
    let mut output = Vec::new();
    let copy = run(&launch, &mut output).unwrap();
    assert_eq!(copy.disposition, ReplayChronologyCopyDisposition::Upgraded);
    assert_eq!(copy.previous_policy, ReplayPolicy::DeterministicPreferred);
    assert_eq!(
        copy.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    assert_eq!(fs::read(&input).unwrap(), before);
    assert!(!reference.exists());
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["mode"], "replay_chronology_upgrade_copy");
    assert_eq!(result["input_modified"], false);
    assert_eq!(result["references_verified"], false);
    assert_eq!(result["previous_policy"], "DeterministicPreferred");
    assert_eq!(result["replay_policy"], "DeterministicCommittedOrderV2");
    assert!(
        result["older_reader_warning"]
            .as_str()
            .unwrap()
            .contains("Older Riotbox versions reject")
    );
    let loaded = load_session_json(&destination).unwrap();
    assert_eq!(loaded.source_refs, session.source_refs);
    assert_eq!(loaded.session_id, session.session_id);
    assert!(
        build_committed_replay_plan(&loaded.action_log)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn already_committed_order_is_reported_as_an_unchanged_policy_copy() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("original.json");
    let destination = dir.path().join("copy.json");
    save_session_json(
        &input,
        &SessionFile::new("generated-v2", "test", "2026-10-16"),
    )
    .unwrap();
    let before = fs::read(&input).unwrap();
    let mut output = Vec::new();
    let copy = run(&launch(&input, &destination), &mut output).unwrap();
    assert_eq!(
        copy.disposition,
        ReplayChronologyCopyDisposition::AlreadyCommittedOrder
    );
    assert_eq!(fs::read(&destination).unwrap(), before);
    assert_eq!(fs::read(&input).unwrap(), before);
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["previous_policy"], result["replay_policy"]);
}

#[test]
fn refused_copy_emits_no_success_and_creates_no_destination() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("original.json");
    let destination = dir.path().join("copy.json");
    let mut session = SessionFile::new("generated-snapshot-only", "test", "2026-10-16");
    session.action_log.replay_policy = ReplayPolicy::SnapshotOnly;
    save_session_json(&input, &session).unwrap();
    let before = fs::read(&input).unwrap();
    let mut output = Vec::new();
    assert!(run(&launch(&input, &destination), &mut output).is_err());
    assert!(output.is_empty());
    assert!(!destination.exists());
    assert_eq!(fs::read(&input).unwrap(), before);
}

struct BrokenOutput {
    fail_flush: bool,
}

impl Write for BrokenOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_flush {
            Ok(buffer.len())
        } else {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "generated stdout failure",
            ))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "generated stdout flush failure",
        ))
    }
}

#[test]
fn stdout_failure_explicitly_acknowledges_the_already_published_copy() {
    for fail_flush in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("original.json");
        let destination = dir.path().join("published.json");
        save_session_json(
            &input,
            &SessionFile::new("generated-broken-output", "test", "2026-10-16"),
        )
        .unwrap();
        let before = fs::read(&input).unwrap();
        let error = run(
            &launch(&input, &destination),
            &mut BrokenOutput { fail_flush },
        )
        .unwrap_err();
        assert!(error.to_string().contains("was published"));
        assert!(error.to_string().contains(destination.to_str().unwrap()));
        assert!(error.to_string().contains("stdout"));
        assert_eq!(fs::read(&destination).unwrap(), before);
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}
