use std::{error::Error, fs, path::Path};

use riotbox_app::jam_app::JamAppState;
use riotbox_core::action::LiveRecordingDuration;
use serde_json::Value;

use super::{
    Report, admit_args, contract::Contract, create_owner, file_hash, measure_windows,
    persist_result, prepare, windows,
};

#[test]
fn preparation_never_accepts_freeform_sources_or_host_options() {
    let valid = [
        "--prepare-generated-live-master-load-v1",
        "--output",
        "/tmp/fresh",
    ]
    .map(String::from);
    assert!(admit_args(&valid).is_ok());
    for option in [
        "--source",
        "--isolated-silent-host",
        "--bpm",
        "--limiter-calibration-v2",
    ] {
        let mut args = valid.to_vec();
        args.push(option.into());
        assert!(admit_args(&args).is_err());
    }
    assert!(admit_args(&valid[..2]).is_err());
    assert!(admit_args(&[]).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn owner_admission_rejects_relative_existing_missing_parent_and_symlink_targets() {
    let parent = tempfile::tempdir().unwrap();
    let occupied = parent.path().join("occupied");
    fs::create_dir(&occupied).unwrap();
    fs::write(occupied.join("sentinel"), b"keep").unwrap();
    assert!(create_owner(Path::new("relative")).is_err());
    assert!(create_owner(&occupied).is_err());
    assert!(create_owner(&parent.path().join("missing/child")).is_err());
    let link = parent.path().join("linked");
    std::os::unix::fs::symlink(&occupied, &link).unwrap();
    assert!(create_owner(&link).is_err());
    assert_eq!(fs::read(occupied.join("sentinel")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(&occupied).unwrap().count(), 1);
    let new = create_owner(&parent.path().join("fresh")).unwrap();
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(new).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn failed_preparation_retains_identity_and_reason_before_error_is_enforced() {
    let owner = tempfile::tempdir().unwrap();
    let mut report = Report::new();
    report.identity = Some(serde_json::json!({"already_computed": true}));
    let failed: Result<(), Box<dyn Error>> = Err("injected preparation failure".into());
    persist_result(owner.path(), &mut report, &failed).unwrap();
    let persisted: Value =
        serde_json::from_slice(&fs::read(owner.path().join("preparation.json")).unwrap()).unwrap();
    assert!(failed.is_err());
    assert_eq!(persisted["result"], "failed");
    assert_eq!(persisted["identity"]["already_computed"], true);
    assert_eq!(persisted["failure_reason"], "injected preparation failure");
    assert_eq!(persisted["host_execution_allowed"], false);
    assert_eq!(persisted["quality_proof"], false);
    let before = fs::read(owner.path().join("preparation.json")).unwrap();
    assert!(persist_result(owner.path(), &mut Report::new(), &Ok(())).is_err());
    assert_eq!(
        fs::read(owner.path().join("preparation.json")).unwrap(),
        before
    );
}

#[test]
fn stale_session_is_rejected_before_restore_or_render() {
    let owner = tempfile::tempdir().unwrap();
    fs::write(owner.path().join("session.json"), b"stale").unwrap();
    let error = windows::measure(
        owner.path(),
        &Contract::load().unwrap(),
        44_100,
        LiveRecordingDuration::TwoBars,
        b"expected",
        "old-file-hash",
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("changed before window"));
    assert_eq!(fs::read_dir(owner.path()).unwrap().count(), 1);
}

#[cfg(target_os = "linux")]
#[test]
fn generated_product_spine_roundtrip_all_six_windows_and_mutation_controls() {
    let parent = tempfile::tempdir().unwrap();
    let root = create_owner(&parent.path().join("prepared")).unwrap();
    let contract = Contract::load().unwrap();
    let mut report = Report::new();
    let result = prepare(&root, &contract, &mut report);
    persist_result(&root, &mut report, &result).unwrap();
    result.unwrap();
    assert_eq!(report.windows.len(), 6);
    assert!(report.windows.iter().all(|window| window.passed));
    let json = serde_json::to_value(&report).unwrap();
    for (case, (rate, bars)) in json["windows"].as_array().unwrap().iter().zip([
        (44_100, 2),
        (44_100, 8),
        (44_100, 16),
        (48_000, 2),
        (48_000, 8),
        (48_000, 16),
    ]) {
        assert_eq!(case["sample_rate_hz"], rate);
        assert_eq!(case["duration_bars"], bars);
        assert_eq!(case["recording_plan_admitted"], true);
        assert_eq!(case["generated_output_geometry"], true);
        assert_eq!(case["pattern_origin"], "fixture");
    }
    assert_eq!(json["result"], "prepared");
    assert_eq!(json["identity"]["save_restore_identity_verified"], true);
    assert_eq!(
        json["identity"]["contributors"].as_array().unwrap().len(),
        5
    );
    assert_eq!(json["quality_proof"], false);
    assert_eq!(json["source_intelligence_qualified"], false);
    assert_eq!(json["host_execution_allowed"], false);
    assert_eq!(json["live_controller_executed"], false);
    assert_eq!(
        json["identity"]["generator_sha256"],
        file_hash(&root.join("generator.py")).unwrap()
    );
    assert_eq!(
        json["identity"]["producer_binary_sha256"],
        file_hash(Path::new("/proc/self/exe")).unwrap()
    );

    let session_path = root.join("session.json");
    let (saved_session, capture_path) = restored_identity(&root);
    let session_hash = file_hash(&session_path).unwrap();
    assert!(capture_path.starts_with(&root));
    for path in [
        root.join("generated-source/dense_break_132.wav"),
        capture_path,
    ] {
        let original = fs::read(&path).unwrap();
        let mut changed = original.clone();
        // Modify PCM only: remains a decodable WAV, but is no longer this identity.
        let last = changed.len() - 1;
        changed[last] ^= 1;
        fs::write(&path, changed).unwrap();
        assert!(
            windows::measure(
                &root,
                &contract,
                44_100,
                LiveRecordingDuration::TwoBars,
                &saved_session,
                &session_hash
            )
            .is_err()
        );
        fs::write(&path, &original).unwrap();
        let absent = path.with_extension("temporarily-absent");
        fs::rename(&path, &absent).unwrap();
        assert!(
            windows::measure(
                &root,
                &contract,
                44_100,
                LiveRecordingDuration::TwoBars,
                &saved_session,
                &session_hash
            )
            .is_err()
        );
        fs::rename(&absent, &path).unwrap();
    }
    // A completed owner cannot be retried or overwritten.
    assert!(create_owner(&root).is_err());
    assert_eq!(file_hash(&session_path).unwrap(), session_hash);
    // The production aggregate seam must retain all six computed collision
    // reports without replacing earlier successful artifacts.
    let before: Vec<_> = json["windows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|window| window["artifact_sha256"].as_str().unwrap().to_owned())
        .collect();
    let mut failed_report = Report::new();
    let rejected = measure_windows(
        &root,
        &contract,
        &saved_session,
        &session_hash,
        &mut failed_report,
    );
    assert!(rejected.is_err());
    assert_eq!(failed_report.windows.len(), 6);
    assert!(failed_report.windows.iter().all(|window| !window.passed));
    let failed_owner = create_owner(&parent.path().join("failed-evidence")).unwrap();
    persist_result(&failed_owner, &mut failed_report, &rejected).unwrap();
    let failed_json: Value =
        serde_json::from_slice(&fs::read(failed_owner.join("preparation.json")).unwrap()).unwrap();
    assert_eq!(failed_json["result"], "failed");
    for (window, hash) in failed_json["windows"]
        .as_array()
        .unwrap()
        .iter()
        .zip(before)
    {
        assert_eq!(
            file_hash(Path::new(window["artifact_path"].as_str().unwrap())).unwrap(),
            hash
        );
        assert!(window["active_samples"].as_u64().unwrap() > 0);
        assert!(window["artifact_error"].is_string());
    }
}

// Keep the large App facade out of the caller's frame while it invokes the
// production prepare/measure paths. Return only the identity that tests need.
#[cfg(target_os = "linux")]
fn restored_identity(root: &Path) -> (Vec<u8>, std::path::PathBuf) {
    let restored =
        JamAppState::from_json_files(root.join("session.json"), Option::<&Path>::None).unwrap();
    assert_eq!(
        restored.session.action_log.replay_policy,
        riotbox_core::session::ReplayPolicy::DeterministicPreferred,
        "the frozen generated V1 control must not silently become V2 evidence"
    );
    assert!(restored.session.export_receipts.is_empty());
    let capture = &restored.session.captures[0];
    assert!(capture.audio_identity.is_some());
    let path = Path::new(&capture.storage_path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    (serde_json::to_vec(&restored.session).unwrap(), path)
}
