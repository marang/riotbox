use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

const OBSERVER: &str =
    include_str!("fixtures/observer_audio_correlation/events_locked_grid.ndjson");
const MANIFEST: &str =
    include_str!("fixtures/observer_audio_correlation/manifest_locked_grid.json");

#[test]
fn manifest_anchor_overflow_degrades_local_report_and_blocks_strict_publication() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer = temp.path().join("observer.ndjson");
    let manifest = temp.path().join("manifest.json");
    fs::write(&observer, OBSERVER).expect("synthetic observer metadata");

    for anchors in overflowing_anchor_counts() {
        let mut metadata: Value = serde_json::from_str(MANIFEST).expect("fixture metadata");
        metadata["source_timing"]["anchor_evidence"] = anchors;
        fs::write(&manifest, serde_json::to_vec(&metadata).expect("JSON"))
            .expect("synthetic manifest metadata");

        let local = run_report(&observer, &manifest, None);
        assert!(local.status.success(), "{:?}", local);
        assert!(local.stderr.is_empty());
        let report: Value = serde_json::from_slice(&local.stdout).expect("degraded JSON report");
        assert!(report["output_path"]["source_timing"].is_null());
        assert_eq!(report["output_path"]["present"], false);
        assert!(
            report["output_path"]["issues"]
                .as_array()
                .expect("issues")
                .contains(&json!("source_timing=malformed"))
        );

        assert_strict_rejection_preserves_output(
            &observer,
            &manifest,
            temp.path(),
            "source_timing=malformed",
        );
    }
}

#[test]
fn observer_anchor_overflow_degrades_local_report_and_blocks_strict_publication() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer = temp.path().join("observer.ndjson");
    let manifest = temp.path().join("manifest.json");
    fs::write(&manifest, MANIFEST).expect("synthetic manifest metadata");

    for anchors in overflowing_anchor_counts() {
        let mut events: Vec<Value> = OBSERVER
            .lines()
            .map(|line| serde_json::from_str(line).expect("fixture event"))
            .collect();
        events[0]["snapshot"]["source_timing"]["anchor_evidence"] = anchors;
        let ndjson = events
            .iter()
            .map(|event| serde_json::to_string(event).expect("JSON"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&observer, ndjson).expect("synthetic observer metadata");

        let local = run_report(&observer, &manifest, None);
        assert!(local.status.success(), "{:?}", local);
        assert!(local.stderr.is_empty());
        let report: Value = serde_json::from_slice(&local.stdout).expect("degraded JSON report");
        assert!(report["control_path"]["observer_source_timing"].is_null());
        assert!(report["output_path"]["source_timing_anchor_alignment"].is_null());

        assert_strict_rejection_preserves_output(
            &observer,
            &manifest,
            temp.path(),
            "malformed observer source timing evidence",
        );
    }
}

fn overflowing_anchor_counts() -> [Value; 2] {
    [(u64::MAX, 1, 0), (u64::MAX - 1, 1, 1)].map(|(kick, backbeat, transient)| {
        json!({
            "primary_anchor_count": u64::MAX,
            "primary_kick_anchor_count": kick,
            "primary_backbeat_anchor_count": backbeat,
            "primary_transient_anchor_count": transient,
        })
    })
}

fn assert_strict_rejection_preserves_output(
    observer: &Path,
    manifest: &Path,
    directory: &Path,
    expected_error: &str,
) {
    let output_path = directory.join("report.json");
    for existing in [false, true] {
        if existing {
            fs::write(&output_path, b"previous verified report").expect("previous report");
        }
        let strict = run_report(observer, manifest, Some(&output_path));
        assert_eq!(strict.status.code(), Some(1), "{:?}", strict);
        assert!(strict.stdout.is_empty());
        let error = String::from_utf8(strict.stderr).expect("error UTF-8");
        assert!(error.contains(expected_error), "{error}");
        assert!(!error.contains("panicked"), "{error}");
        if existing {
            assert_eq!(
                fs::read(&output_path).expect("previous report"),
                b"previous verified report"
            );
            fs::remove_file(&output_path).expect("remove test report");
        } else {
            assert!(!output_path.exists());
        }
    }
}

fn run_report(observer: &Path, manifest: &Path, strict_output: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_observer_audio_correlate"));
    command
        .arg("--observer")
        .arg(observer)
        .arg("--manifest")
        .arg(manifest)
        .arg("--json");
    if let Some(output) = strict_output {
        command
            .arg("--require-evidence")
            .arg("--output")
            .arg(output);
    }
    command
        .output()
        .expect("run metadata-only Observer correlation")
}
