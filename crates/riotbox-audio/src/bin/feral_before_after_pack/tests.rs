use super::args::Args;
use super::config::{CHANNEL_COUNT, MIN_AFTER_RMS, MIN_DELTA_RMS, PACK_ID, SAMPLE_RATE};
use super::mix::signal_delta_metrics;
use super::pack_builder::render_pack;
use super::source_window::seconds_to_frames;
use riotbox_audio::{
    listening_manifest::LISTENING_MANIFEST_SCHEMA_VERSION,
    runtime::signal_metrics,
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use std::{fs, path::PathBuf};

#[test]
fn parses_required_source_and_custom_output() {
    let parsed = Args::parse([
        "--source".to_string(),
        "input.wav".to_string(),
        "--output-dir".to_string(),
        "out".to_string(),
        "--duration-seconds".to_string(),
        "0.5".to_string(),
        "--source-window-seconds".to_string(),
        "0.25".to_string(),
    ])
    .expect("parse args");

    assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
    assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
    assert_eq!(parsed.duration_seconds, 0.5);
    assert_eq!(parsed.source_window_seconds, 0.25);
}

#[test]
fn rejects_missing_source() {
    assert!(Args::parse(Vec::<String>::new()).is_err());
}

#[test]
fn renders_pack_files_and_distinct_after_audio() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_path = temp.path().join("source.wav");
    let output_dir = temp.path().join("pack");
    write_interleaved_pcm16_wav(
        &source_path,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &synthetic_break_source(seconds_to_frames(0.5)),
    )
    .expect("write source");

    let args = Args {
        source_path,
        output_dir: Some(output_dir.clone()),
        date: "test".into(),
        source_start_seconds: 0.0,
        duration_seconds: 0.5,
        source_window_seconds: 0.25,
        show_help: false,
    };

    render_pack(&args).expect("render pack");

    assert!(output_dir.join("01_source_excerpt.wav").is_file());
    assert!(output_dir.join("02_riotbox_feral_changed.wav").is_file());
    assert!(output_dir.join("03_before_then_after.wav").is_file());
    assert!(output_dir.join("stems/w30_source_chop.wav").is_file());
    assert!(output_dir.join("stems/tr909_fill.wav").is_file());
    assert!(output_dir.join("stems/mc202_instigator.wav").is_file());
    assert!(output_dir.join("comparison.md").is_file());
    assert!(output_dir.join("manifest.json").is_file());

    let source = SourceAudioCache::load_pcm_wav(output_dir.join("01_source_excerpt.wav"))
        .expect("load source");
    let after = SourceAudioCache::load_pcm_wav(output_dir.join("02_riotbox_feral_changed.wav"))
        .expect("load after");
    let source_metrics = signal_metrics(source.interleaved_samples());
    let after_metrics = signal_metrics(after.interleaved_samples());
    let delta = signal_delta_metrics(source.interleaved_samples(), after.interleaved_samples());

    assert!(source_metrics.rms > 0.001);
    assert!(after_metrics.rms > 0.001);
    assert!(delta.rms > 0.005);

    let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
    assert_eq!(
        manifest["schema_version"],
        LISTENING_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(manifest["pack_id"], PACK_ID);
    assert_eq!(manifest["result"], "pass");
    assert_eq!(
        manifest["artifacts"].as_array().expect("artifacts").len(),
        8
    );
    assert!(
        manifest["metrics"]["riotbox_after"]["rms"]
            .as_f64()
            .expect("after rms")
            > f64::from(MIN_AFTER_RMS)
    );
    assert!(
        manifest["metrics"]["source_after_delta"]["rms"]
            .as_f64()
            .expect("delta rms")
            > f64::from(MIN_DELTA_RMS)
    );
    for artifact in manifest["artifacts"].as_array().expect("artifacts") {
        let path = PathBuf::from(artifact["path"].as_str().expect("artifact path"));
        assert!(path.is_file(), "{} missing", path.display());
        if let Some(metrics_path) = artifact["metrics_path"].as_str() {
            let metrics_path = PathBuf::from(metrics_path);
            assert!(metrics_path.is_file(), "{} missing", metrics_path.display());
        }
    }
}

fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
    for frame in 0..frame_count {
        let phase = frame as f32 / SAMPLE_RATE as f32;
        let beat = frame % 11_025;
        let kick = if beat < 1_200 {
            ((1.0 - beat as f32 / 1_200.0).max(0.0) * 0.9)
                * (phase * 80.0 * std::f32::consts::TAU).sin()
        } else {
            0.0
        };
        let grit = (phase * 730.0 * std::f32::consts::TAU).sin() * 0.08;
        let sample = kick + grit;
        samples.push(sample);
        samples.push(sample * 0.96);
    }
    samples
}
