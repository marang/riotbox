use std::{
    error::Error,
    io::{Read, Write},
};

use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, OfflineAudioMetrics, RuntimeMixRenderPlan,
        SourceMonitorRenderState, limiter_calibration,
    },
    source_audio::SourceAudioCache,
};
use riotbox_core::{
    action::SourceMonitorMode,
    source_graph::{
        DecodeProfile, GraphProvenance, ManualSourceTimingGrid, SourceDescriptor, SourceGraph,
        install_manual_source_timing_grid,
    },
};
use serde_json::{Value, json};

use super::protocol::{CalibrationVersion, HistoricalControls, pcm_hash};
use super::{
    CHANNELS, Case, ComparisonFailure, MAX_REQUEST_BYTES, SAMPLE_RATE, bits_equal, compare_plan,
    compare_plan_with_history, failed, fourfold_condition, metrics_json, policy_name, read_request,
    render_diagnostics, render_once, report_value, run, validate_samples, write_failure_record,
};

#[test]
fn every_registered_historical_control_matches_the_repository_v2_protocol() {
    // Compile-time repository metadata only: never follow any artifact/source
    // paths contained in this preregistration document.
    let protocol: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/benchmarks/master_bus_limiter_calibration_protocol_v2.json"
    )))
    .unwrap();
    assert_eq!(
        protocol["schema"],
        "riotbox.master_bus_limiter_calibration.v2"
    );
    let cases = protocol["cases"].as_array().unwrap();
    let registered_cases = [Case::Dense, Case::Tonal, Case::Sparse];
    assert_eq!(cases.len(), registered_cases.len());
    for (case, pinned) in registered_cases.into_iter().zip(cases) {
        assert_eq!(
            serde_json::from_value::<Case>(pinned["case_id"].clone()).unwrap(),
            case
        );
        let controls = HistoricalControls::registered(case);
        let pinned_clean = pinned["historical_controls"]["clean_pcm_sha256_f32le"]
            .as_str()
            .unwrap();
        let pinned_doubled = pinned["historical_controls"]["stress_2x_pcm_sha256_f32le"]
            .as_str()
            .unwrap();
        assert_eq!(controls.pre, pinned_clean, "{case:?}: pre/clean pin");
        for (policy, actual) in ["A", "B", "C"].into_iter().zip(&controls.stress_2x) {
            assert_eq!(actual, pinned_doubled, "{case:?}: 2x policy {policy} pin");
        }
        let diagnostics = controls.diagnostics(&controls);
        assert_eq!(
            diagnostics["reference_report_sha256"],
            protocol["historical_basis"]["report_sha256"]
        );
        let pinned_diagnostic = json!({
            "pre_sha256_f32le": pinned_clean,
            "stress_2x_sha256_f32le": [pinned_doubled, pinned_doubled, pinned_doubled],
        });
        assert_eq!(diagnostics["expected"], pinned_diagnostic);
        assert_eq!(diagnostics["actual"], pinned_diagnostic);
    }
}

fn metadata_request() -> Value {
    let (path, hash, source_id) = Case::Tonal.identity();
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: source_id.into(),
            path: path.into(),
            content_hash: hash.into(),
            duration_seconds: 4.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "synthetic-metadata-test".into(),
            provider_set: Vec::new(),
            generated_at: "synthetic".into(),
            source_hash: hash.into(),
            analysis_seed: 1,
            run_notes: None,
        },
    );
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 120.0,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    // Deliberately not real audio: only the metadata reader may admit this;
    // the in-memory constructor must reject its content hash.
    json!({"case_id": Case::Tonal, "graph": graph, "source_wav_bytes": [1],
        "output_dir": "/unused-synthetic-output"})
}

#[test]
fn unknown_cases_and_fields_fail_before_any_output_io() {
    assert!(serde_json::from_str::<Case>("\"holdout\"").is_err());
    assert!(read_request(b"{}".as_slice()).is_err());
    for version in [CalibrationVersion::V1, CalibrationVersion::V2] {
        assert!(run(b"{\"case_id\":\"holdout\"}".as_slice(), Vec::new(), version).is_err());
    }
    let mut request = metadata_request();
    request["extra_field"] = json!(true);
    assert!(read_request(serde_json::to_vec(&request).unwrap().as_slice()).is_err());
}

#[test]
fn graph_identity_format_and_timing_guards_are_metadata_only() {
    let encoded = serde_json::to_vec(&metadata_request()).unwrap();
    let request = read_request(encoded.as_slice()).unwrap();
    let mutations: [fn(&mut SourceGraph); 5] = [
        |graph| graph.source.path = "unregistered.wav".into(),
        |graph| graph.source.content_hash = "sha256:wrong".into(),
        |graph| graph.provenance.source_hash = "sha256:wrong".into(),
        |graph| graph.source.sample_rate = 48_000,
        |graph| graph.timing.primary_hypothesis_id = None,
    ];
    for mutate in mutations {
        let mut graph = request.graph.clone();
        mutate(&mut graph);
        assert!(Case::Tonal.validate_graph(&graph).is_err());
    }
    assert!(Case::Dense.validate_graph(&request.graph).is_err());
    let directory = tempfile::tempdir().unwrap();
    let mut value = metadata_request();
    value["output_dir"] = json!(directory.path());
    let error = run(
        serde_json::to_vec(&value).unwrap().as_slice(),
        Vec::new(),
        CalibrationVersion::V1,
    )
    .unwrap_err();
    assert!(error.to_string().contains("hash"), "{error}");
    assert!(directory.path().read_dir().unwrap().next().is_none());
}

#[test]
fn bounded_reader_rejects_oversized_request() {
    let input = std::io::repeat(b' ').take((MAX_REQUEST_BYTES + 10) as u64);
    assert!(
        read_request(input)
            .err()
            .unwrap()
            .to_string()
            .contains("32 MiB")
    );
}

#[test]
fn bit_parity_rejects_signed_zero_and_bad_alignment() {
    assert!(!bits_equal(&[0.0], &[-0.0]));
    assert!(!bits_equal(&[0.0], &[]));
    assert!(validate_samples(&[0.0, f32::NAN], 1).is_err());
    assert!(validate_samples(&[0.0], 1).is_err());
    assert!(validate_samples(&[], 0).is_err());
    assert!(validate_samples(&[0.1, -0.1], 1).is_ok());
}

#[test]
fn silence_and_nonfinite_metrics_do_not_become_success_json() {
    assert!(compare_plan(&RuntimeMixRenderPlan::default(), 16, Case::Tonal).is_err());
    assert!(
        metrics_json(OfflineAudioMetrics {
            rms: f32::INFINITY,
            ..Default::default()
        })
        .is_err()
    );
}

fn synthetic_monitor_plan(amplitude: f32) -> RuntimeMixRenderPlan {
    synthetic_monitor_samples(vec![amplitude; 2_048])
}

fn synthetic_monitor_samples(samples: Vec<f32>) -> RuntimeMixRenderPlan {
    let cache = SourceAudioCache::from_interleaved_samples(
        "synthetic-no-file.wav",
        SAMPLE_RATE,
        CHANNELS,
        samples,
    )
    .unwrap();
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: 0.0,
        },
        source_monitor_render: SourceMonitorRenderState {
            source_anchor_seconds: Some(0.0),
            ..SourceMonitorRenderState::from_source_cache(SourceMonitorMode::Source, Some(&cache))
        },
        ..Default::default()
    }
}

#[test]
fn synthetic_comparison_preserves_clean_and_limits_exact_doubled_stress() {
    let result = compare_plan(&synthetic_monitor_plan(0.8), 514, Case::Tonal).unwrap();
    assert_eq!(result["pre_samples"].as_array().unwrap().len(), 1_028);
    assert_eq!(result["controls"]["baseline_api_bit_exact"], true);
    assert_eq!(result["controls"]["repeat_128_bit_exact"], true);
    assert_eq!(result["controls"]["partition_257_bit_exact"], true);
    let pre_peak = result["conditions"][0]["outputs"][0]["limiter"]["pre"]["peak_abs"]
        .as_f64()
        .unwrap();
    for output in result["conditions"][0]["outputs"].as_array().unwrap() {
        assert_eq!(output["samples"], result["pre_samples"]);
        assert_eq!(output["limiter"]["applied"], false);
    }
    for output in result["conditions"][1]["outputs"].as_array().unwrap() {
        assert_eq!(
            output["limiter"]["pre"]["peak_abs"].as_f64().unwrap(),
            pre_peak * 2.0
        );
        assert_eq!(output["limiter"]["applied"], true);
    }
}

#[test]
fn clean_failure_retains_actual_reports_without_running_stress() {
    let error = compare_plan(&synthetic_monitor_plan(1.3), 514, Case::Tonal)
        .map(|_| ())
        .expect_err("synthetic overload must fail the clean gate");
    let failure = error.downcast_ref::<ComparisonFailure>().unwrap();
    let mut record = Vec::new();
    let preparation = json!({"recipe": "synthetic-test", "committed_actions": [], "capture_window": {"start_frame": 0, "end_frame": 88_200}});
    write_failure_record(
        failure,
        "comparison",
        Some(preparation.clone()),
        &mut record,
    )
    .unwrap();
    assert!(record.len() < 64 * 1024);
    let line = String::from_utf8(record).unwrap();
    let envelope: Value = serde_json::from_str(
        line.trim()
            .strip_prefix("RIOTBOX_LIMITER_CALIBRATION_FAILURE ")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(envelope["stage"], "clean_gate");
    assert_eq!(envelope["preparation"], preparation);
    let reports = &envelope["diagnostics"];
    for name in ["primary_128", "repeat_128", "partition_257"] {
        assert!(
            reports[name]["limiter"]["pre"]["peak_abs"]
                .as_f64()
                .unwrap()
                > 1.0
        );
        assert!(
            reports[name]["limiter"]["limited_sample_count"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert!(
            reports[name]["limiter"]["post"]["peak_abs"]
                .as_f64()
                .unwrap()
                < 1.0
        );
    }
    for name in ["clean_A", "clean_B", "clean_C"] {
        assert!(reports[name]["limited_sample_count"].as_u64().unwrap() > 0);
    }
    assert!(reports.get("stress_2x_A").is_none());
    assert!(!line.contains("\"samples\""));
}

#[test]
fn existing_activity_and_sparse_rms_gates_stop_before_stress() {
    for (amplitude, case, stage) in [
        (1.0e-8, Case::Tonal, "clean_silence"),
        (0.005, Case::Sparse, "clean_sparse_rms"),
    ] {
        let error = compare_plan(&synthetic_monitor_plan(amplitude), 514, case)
            .map(|_| ())
            .expect_err("synthetic weak interval must fail the existing recipe gate");
        let failure = error.downcast_ref::<ComparisonFailure>().unwrap();
        assert_eq!(failure.stage, stage);
        assert!(
            failure.diagnostics["clean_A"]["pre"]["peak_abs"]
                .as_f64()
                .unwrap()
                > 0.0
        );
        assert!(failure.diagnostics.get("stress_2x_A").is_none());
    }
}

fn samples_from_json(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|sample| sample.as_f64().unwrap() as f32)
        .collect()
}

fn synthetic_history(v1: &Value) -> HistoricalControls {
    HistoricalControls {
        pre: pcm_hash(&samples_from_json(&v1["pre_samples"])),
        stress_2x: std::array::from_fn(|index| {
            pcm_hash(&samples_from_json(
                &v1["conditions"][1]["outputs"][index]["samples"],
            ))
        }),
    }
}

#[test]
fn v2_preserves_all_v1_controls_and_adds_only_direct_fourfold_outputs() {
    let plan = synthetic_monitor_plan(0.3);
    let v1 = compare_plan(&plan, 514, Case::Tonal).unwrap();
    assert!(v1.get("protocol_version").is_none());
    assert_eq!(v1.as_object().unwrap().len(), 6);
    assert_eq!(v1["controls"].as_object().unwrap().len(), 3);
    assert_eq!(v1["conditions"].as_array().unwrap().len(), 2);
    let v2 =
        compare_plan_with_history(&plan, 514, Case::Tonal, Some(&synthetic_history(&v1))).unwrap();
    assert_eq!(v2["protocol_version"], "v2");
    assert_eq!(v2["conditions"].as_array().unwrap().len(), 3);
    assert_eq!(v2["conditions"][2]["condition"], "stress_4x");
    let mut without_v2 = v2.clone();
    without_v2
        .as_object_mut()
        .unwrap()
        .remove("protocol_version");
    without_v2["conditions"].as_array_mut().unwrap().pop();
    assert_eq!(
        serde_json::to_vec(&without_v2).unwrap(),
        serde_json::to_vec(&v1).unwrap()
    );

    let pre = samples_from_json(&v1["pre_samples"]);
    let exact_fourfold: Vec<f32> = pre.iter().map(|sample| *sample * 4.0_f32).collect();
    let expected = limiter_calibration::compare(&exact_fourfold).unwrap();
    for (index, output) in expected.iter().enumerate() {
        assert_eq!(
            v2["conditions"][2]["outputs"][index]["limiter"],
            report_value(output.limiter)
        );
        assert!(bits_equal(
            &samples_from_json(&v2["conditions"][2]["outputs"][index]["samples"]),
            &output.samples
        ));
        assert!(
            !v2["conditions"][1]["outputs"][index]["limiter"]["applied"]
                .as_bool()
                .unwrap()
        );
        assert!(output.limiter.applied);
    }
}

#[test]
fn historical_mismatch_retains_clean_and_doubled_reports_before_any_fourfold() {
    let plan = synthetic_monitor_plan(0.8);
    let v1 = compare_plan(&plan, 514, Case::Tonal).unwrap();
    for mismatched_control in 0..4 {
        let mut expected = synthetic_history(&v1);
        if mismatched_control == 0 {
            expected.pre = "mismatched-generated-pre".into();
        } else {
            expected.stress_2x[mismatched_control - 1] = "mismatched-generated-policy".into();
        }
        let error = compare_plan_with_history(&plan, 514, Case::Tonal, Some(&expected))
            .map(|_| ())
            .expect_err("every historical control is mandatory");
        let failure = error.downcast_ref::<ComparisonFailure>().unwrap();
        assert_eq!(failure.stage, "historical_controls");
        for policy in ["A", "B", "C"] {
            assert!(failure.diagnostics.get(format!("clean_{policy}")).is_some());
            assert!(
                failure
                    .diagnostics
                    .get(format!("stress_2x_{policy}"))
                    .is_some()
            );
            assert!(
                failure
                    .diagnostics
                    .get(format!("stress_4x_{policy}"))
                    .is_none()
            );
        }
        assert_eq!(
            failure.diagnostics["historical_v1_controls"]["actual"]["pre_sha256_f32le"],
            synthetic_history(&v1).pre
        );
        assert_eq!(
            failure.diagnostics["historical_v1_controls"]["expected"]["pre_sha256_f32le"],
            expected.pre
        );
        assert_failure_envelope(error.as_ref());
    }
    // The real closed case table also rejects a synthetic plan, with no knob to
    // admit these test references through the public request/CLI route.
    let error = compare_plan_with_history(
        &plan,
        514,
        Case::Tonal,
        Some(&HistoricalControls::registered(Case::Tonal)),
    )
    .map(|_| ())
    .expect_err("generated PCM cannot match a registered V1 control");
    assert_eq!(
        error.downcast_ref::<ComparisonFailure>().unwrap().stage,
        "historical_controls"
    );
}

#[test]
fn fourfold_uses_original_pre_even_when_doubled_control_was_limited() {
    let plan = synthetic_monitor_plan(0.8);
    let v1 = compare_plan(&plan, 514, Case::Tonal).unwrap();
    let v2 =
        compare_plan_with_history(&plan, 514, Case::Tonal, Some(&synthetic_history(&v1))).unwrap();
    let raw_pre = samples_from_json(&v1["pre_samples"]);
    let fourfold: Vec<f32> = raw_pre.iter().map(|sample| *sample * 4.0_f32).collect();
    for (index, expected) in limiter_calibration::compare(&fourfold)
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(
            v1["conditions"][1]["outputs"][index]["limiter"]["applied"],
            true
        );
        assert!(bits_equal(
            &samples_from_json(&v2["conditions"][2]["outputs"][index]["samples"]),
            &expected.samples
        ));
        assert_eq!(
            v2["conditions"][2]["outputs"][index]["limiter"],
            report_value(expected.limiter)
        );
        let limited_twice: Vec<f32> =
            samples_from_json(&v1["conditions"][1]["outputs"][index]["samples"])
                .into_iter()
                .map(|sample| sample * 2.0_f32)
                .collect();
        let wrong = limiter_calibration::compare(&limited_twice).unwrap();
        // Both post buffers can saturate to the same ceiling; the retained pre
        // report must still prove the raw 4x input, not twice-limited control PCM.
        assert_ne!(expected.limiter.pre, wrong[index].limiter.pre);
    }
}

fn assert_failure_envelope(error: &(dyn Error + 'static)) {
    let preparation = json!({"recipe": "synthetic-test", "committed_actions": [], "capture_window": {"start_frame": 0, "end_frame": 88_200}});
    let mut bytes = Vec::new();
    write_failure_record(error, "comparison", Some(preparation.clone()), &mut bytes).unwrap();
    assert!(bytes.len() < 64 * 1024);
    let line = String::from_utf8(bytes).unwrap();
    let value: Value = serde_json::from_str(
        line.trim()
            .strip_prefix("RIOTBOX_LIMITER_CALIBRATION_FAILURE ")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["preparation"], preparation);
    assert!(!line.contains("\"samples\""));
    assert!(!line.contains("\"graph\""));
}

#[test]
fn fourfold_failure_retains_prior_and_available_fourfold_reports_with_preparation() {
    let clean_pre = vec![0.3; 1_028];
    let mut diagnostics = json!({});
    for (prefix, samples) in [
        ("clean", clean_pre.clone()),
        ("stress_2x", vec![0.6; 1_028]),
    ] {
        for output in limiter_calibration::compare(&samples).unwrap() {
            diagnostics[format!("{prefix}_{}", policy_name(output.policy))] =
                report_value(output.limiter);
        }
    }
    let error = fourfold_condition(&[f32::NAN, 0.0], 1, &mut diagnostics)
        .map(|_| ())
        .expect_err("nonfinite fourfold input must retain prior reports");
    let failure = error.downcast_ref::<ComparisonFailure>().unwrap();
    assert_eq!(failure.stage, "stress_4x_comparison");
    assert_eq!(failure.diagnostics, diagnostics);
    assert_failure_envelope(error.as_ref());
    let error = fourfold_condition(&clean_pre, 515, &mut diagnostics)
        .map(|_| ())
        .expect_err("misaligned output must retain completed fourfold reports");
    let failure = error.downcast_ref::<ComparisonFailure>().unwrap();
    assert_eq!(failure.stage, "stress_4x_output_validation");
    for policy in ["A", "B", "C"] {
        for prefix in ["clean", "stress_2x", "stress_4x"] {
            assert!(
                failure
                    .diagnostics
                    .get(format!("{prefix}_{policy}"))
                    .is_some()
            );
        }
        assert!(
            failure.diagnostics[format!("stress_4x_{policy}")]["limited_sample_count"]
                .as_u64()
                .unwrap()
                > 0
        );
    }
    assert_failure_envelope(error.as_ref());
}

#[test]
fn full_v2_failure_evidence_and_compacted_preparation_fit_stderr_budget() {
    let plan = synthetic_monitor_plan(0.3);
    let primary = render_once(&plan, 514, 128).unwrap();
    let mut diagnostics = json!({});
    for name in ["primary_128", "repeat_128", "partition_257"] {
        diagnostics[name] = render_diagnostics(&primary);
    }
    for (prefix, factor) in [("clean", 1.0), ("stress_2x", 2.0), ("stress_4x", 4.0)] {
        let input: Vec<f32> = primary
            .pre_samples
            .iter()
            .map(|sample| *sample * factor)
            .collect();
        for output in limiter_calibration::compare(&input).unwrap() {
            diagnostics[format!("{prefix}_{}", policy_name(output.policy))] =
                report_value(output.limiter);
        }
    }
    let historical = HistoricalControls::registered(Case::Tonal);
    diagnostics["historical_v1_controls"] = historical.diagnostics(&historical);
    let error = failed(
        "stress_4x_output_validation",
        "synthetic failure",
        &diagnostics,
    );
    let preparation = json!({
        "committed_actions": [{"id": "synthetic-action", "command": "synthetic", "status": "Committed"}],
        "commit_records": [{"action_id": "synthetic-action", "commit_sequence": 1}],
        "capture_window": {"start_frame": 0, "end_frame": 88_200},
        "synthetic_unexpected_metadata": "x".repeat(64 * 1024),
    });
    let mut bytes = Vec::new();
    write_failure_record(error.as_ref(), "comparison", Some(preparation), &mut bytes).unwrap();
    assert!(bytes.len() < 64 * 1024);
    let line = String::from_utf8(bytes).unwrap();
    let record: Value = serde_json::from_str(
        line.trim()
            .strip_prefix("RIOTBOX_LIMITER_CALIBRATION_FAILURE ")
            .unwrap(),
    )
    .unwrap();
    let serialized_diagnostics: Value =
        serde_json::from_slice(&serde_json::to_vec(&diagnostics).unwrap()).unwrap();
    assert!(
        record["diagnostics"] == serialized_diagnostics,
        "all report fields survive bounded serialization"
    );
    assert_eq!(record["preparation"]["truncated"], true);
    assert_eq!(
        record["preparation"]["actions"][0]["id"],
        "synthetic-action"
    );
    assert_eq!(record["preparation"]["capture_window"]["end_frame"], 88_200);
    assert!(!line.contains("\"samples\""));
}

#[test]
fn f32le_hash_binds_bits_including_signed_zero() {
    use sha2::{Digest, Sha256};
    let samples = [0.0_f32, -0.0, 0.3, -0.3];
    let bytes: Vec<u8> = samples.into_iter().flat_map(f32::to_le_bytes).collect();
    assert_eq!(pcm_hash(&samples), format!("{:x}", Sha256::digest(bytes)));
    assert_ne!(pcm_hash(&[0.0]), pcm_hash(&[-0.0]));
}

#[test]
fn maximum_interval_nine_outputs_with_long_json_numbers_fits_v2_stdout_budget() {
    const FRAMES: usize = 192_000;
    // Constant monitor PCM avoids interpolation/position effects: this test
    // bounds serialization, not Source Monitor resampling. Leave an EOF margin.
    let samples = vec![f32::from_bits(0x3eb0_0001); FRAMES * usize::from(CHANNELS) + 2_048];
    let plan = synthetic_monitor_samples(samples);
    // Obtain generated references directly, without a second retained JSON pack.
    let primary = render_once(&plan, FRAMES, 128).unwrap();
    let doubled: Vec<f32> = primary
        .pre_samples
        .iter()
        .map(|sample| *sample * 2.0_f32)
        .collect();
    let doubled_outputs = limiter_calibration::compare(&doubled).unwrap();
    let expected = HistoricalControls {
        pre: pcm_hash(&primary.pre_samples),
        stress_2x: std::array::from_fn(|index| pcm_hash(&doubled_outputs[index].samples)),
    };
    let result = compare_plan_with_history(&plan, FRAMES, Case::Tonal, Some(&expected)).unwrap();
    assert_eq!(result["pre_samples"].as_array().unwrap().len(), FRAMES * 2);
    assert!(result["pre_samples"][0].to_string().len() > 15);
    for condition in result["conditions"].as_array().unwrap() {
        assert_eq!(condition["outputs"].as_array().unwrap().len(), 3);
        for output in condition["outputs"].as_array().unwrap() {
            assert_eq!(output["samples"].as_array().unwrap().len(), FRAMES * 2);
        }
    }
    #[derive(Default)]
    struct ByteCounter(usize);
    impl Write for ByteCounter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = ByteCounter::default();
    serde_json::to_writer(&mut output, &result).unwrap();
    assert!(
        output.0 > 64 * 1024 * 1024,
        "exercise the expanded bound: {}",
        output.0
    );
    assert!(output.0 < 128 * 1024 * 1024, "{}", output.0);
    eprintln!("generated V2 maximum-interval JSON: {} bytes", output.0);
}
