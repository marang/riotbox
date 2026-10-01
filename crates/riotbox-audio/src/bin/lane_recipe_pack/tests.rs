use super::args::Args;
use super::case_catalog::pack_cases;
use super::config::BEATS_PER_BAR;
use super::config::CHANNEL_COUNT;
use super::config::DEFAULT_BPM;
use super::config::PACK_ID;
use super::config::SAMPLE_RATE;
use super::mc202_source_phrase_slot::MC202_SOURCE_PHRASE_SLOT_CONTRACT;
use super::pack_builder::render_pack;
use super::render_case::render_pair;
use super::signal_delta::signal_delta_metrics;
use riotbox_audio::listening_manifest::LISTENING_MANIFEST_SCHEMA_VERSION;
use riotbox_audio::runtime::signal_metrics_with_grid;
use std::fs;
use std::path::PathBuf;

#[test]
fn parses_default_args() {
    let args = Args::parse(Vec::<String>::new()).expect("parse args");

    assert_eq!(args.date, "local");
    assert_eq!(args.output_dir, None);
    assert_eq!(args.duration_seconds, 2.0);
    assert!(!args.show_help);
    assert_eq!(
        args.output_dir(),
        PathBuf::from("artifacts/audio_qa/local").join(PACK_ID)
    );
}

#[test]
fn parses_custom_args() {
    let args = Args::parse([
        "--date".to_string(),
        "audit".to_string(),
        "--duration-seconds".to_string(),
        "1.5".to_string(),
        "--output-dir".to_string(),
        "tmp/pack".to_string(),
    ])
    .expect("parse args");

    assert_eq!(args.date, "audit");
    assert_eq!(args.duration_seconds, 1.5);
    assert_eq!(args.output_dir(), PathBuf::from("tmp/pack"));
}

#[test]
fn rejects_invalid_duration() {
    assert!(Args::parse(["--duration-seconds".to_string(), "0".to_string()]).is_err());
}

#[test]
fn pack_cases_produce_distinct_audio_metrics() {
    let cases = pack_cases();

    assert_eq!(cases.len(), 8);
    for case in cases {
        let (baseline, candidate) = render_pair(&case.render_pair, 88_200);
        let baseline_metrics = signal_metrics_with_grid(
            &baseline,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            DEFAULT_BPM,
            BEATS_PER_BAR,
        );
        let candidate_metrics = signal_metrics_with_grid(
            &candidate,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            DEFAULT_BPM,
            BEATS_PER_BAR,
        );
        let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);

        assert!(
            baseline_metrics.active_samples > 0,
            "{} baseline silent",
            case.id
        );
        assert!(
            candidate_metrics.active_samples > 0,
            "{} candidate silent",
            case.id
        );
        assert!(
            (baseline_metrics.rms - candidate_metrics.rms).abs() >= case.min_rms_delta,
            "{} did not produce required RMS delta {}",
            case.id,
            case.min_rms_delta
        );
        assert!(
            signal_delta_metrics.rms >= case.min_signal_delta_rms,
            "{} did not produce required signal delta RMS {}",
            case.id,
            case.min_signal_delta_rms
        );
        assert!(
            baseline_metrics.onset_count > 0,
            "{} baseline has no detected onsets",
            case.id
        );
        assert!(
            candidate_metrics.event_density_per_bar > 0.0,
            "{} candidate has no event density",
            case.id
        );
    }
}

#[test]
fn render_pack_writes_machine_readable_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("lane-pack");
    let args = Args {
        date: "manifest-smoke".into(),
        output_dir: Some(output_dir.clone()),
        duration_seconds: 2.0,
        show_help: false,
    };

    render_pack(&args).expect("render pack");

    assert!(output_dir.join("pack-summary.md").is_file());
    assert!(output_dir.join("manifest.json").is_file());

    let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");

    assert_eq!(
        manifest["schema_version"],
        LISTENING_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(manifest["pack_id"], PACK_ID);
    assert_eq!(manifest["date"], "manifest-smoke");
    assert_eq!(manifest["result"], "pass");
    assert_eq!(manifest["case_count"], 8);
    assert_eq!(
        manifest["primitive_renderer_boundary"]["schema"],
        "riotbox.primitive_renderer_boundary.v1"
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["evidence_role"],
        "non_product_diagnostic_control"
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["product_output_allowed"],
        false
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["quality_proof"],
        false
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["demo_readiness"],
        "unverified"
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["promotion_blocked"],
        true
    );
    assert_eq!(
        manifest["primitive_renderer_boundary"]["affected_paths"]
            .as_array()
            .expect("primitive affected paths")
            .len(),
        8
    );

    let cases = manifest["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 8);
    let first_case = &cases[0];
    assert_eq!(first_case["id"], "tr909-support-to-fill");
    assert_eq!(first_case["pattern_origin"], "primitive_renderer");
    assert_eq!(
        first_case["evidence_role"],
        "non_product_diagnostic_control"
    );
    assert_eq!(first_case["product_output_allowed"], false);
    assert_eq!(first_case["quality_proof"], false);
    assert_eq!(first_case["demo_readiness"], "unverified");
    assert_eq!(first_case["promotion_blocked"], true);
    assert_eq!(first_case["result"], "pass");
    assert!(
        first_case["metrics"]["baseline"]["rms"]
            .as_f64()
            .expect("baseline rms")
            > 0.0
    );
    assert!(
        first_case["metrics"]["candidate"]["rms"]
            .as_f64()
            .expect("candidate rms")
            > 0.0
    );
    assert!(
        first_case["metrics"]["candidate"]["event_density_per_bar"]
            .as_f64()
            .expect("candidate event density")
            > 0.0
    );
    assert!(
        first_case["metrics"]["signal_delta"]["rms"]
            .as_f64()
            .expect("signal delta rms")
            >= first_case["thresholds"]["min_signal_delta_rms"]
                .as_f64()
                .expect("min signal delta")
    );
    assert!(first_case["metrics"]["mc202_phrase_grid"].is_null());

    let mc202_case = cases
        .iter()
        .find(|case| case["id"] == "mc202-touch-low-to-high")
        .expect("mc202 case");
    assert_eq!(mc202_case["pattern_origin"], "primitive_renderer");
    assert_eq!(mc202_case["metrics"]["mc202_phrase_grid"]["passed"], true);
    assert_eq!(
        mc202_case["metrics"]["mc202_source_phrase_slot"]["passed"],
        true
    );
    assert_eq!(
        mc202_case["metrics"]["mc202_source_phrase_slot"]["contract"],
        MC202_SOURCE_PHRASE_SLOT_CONTRACT
    );
    assert_eq!(
        mc202_case["metrics"]["mc202_source_phrase_slot"]["phrase_index"],
        3
    );
    assert_eq!(
        mc202_case["metrics"]["mc202_phrase_grid"]["resolution"],
        "sixteenth"
    );
    assert!(
        mc202_case["metrics"]["mc202_phrase_grid"]["starts_on_phrase_boundary"]
            .as_bool()
            .expect("phrase boundary")
    );
    assert!(
        mc202_case["metrics"]["mc202_phrase_grid"]["hit_ratio"]
            .as_f64()
            .expect("hit ratio")
            >= 0.95
    );

    let artifacts = manifest["artifacts"].as_array().expect("artifacts");
    assert_eq!(artifacts.len(), 25);
    for artifact in artifacts {
        let path = PathBuf::from(artifact["path"].as_str().expect("artifact path"));
        assert!(path.is_file(), "{} missing", path.display());
        if let Some(metrics_path) = artifact["metrics_path"].as_str() {
            let metrics_path = PathBuf::from(metrics_path);
            assert!(metrics_path.is_file(), "{} missing", metrics_path.display());
        }
    }
}
