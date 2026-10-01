use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::{SourceAudioCache, write_interleaved_pcm16_wav};

// Independent literal CLI contract, not the renderer's output planner.
const ARTIFACTS: [&str; 19] = [
    "stems/01_tr909_beat_fill.wav",
    "stems/01_tr909_beat_fill.metrics.md",
    "stems/02_w30_feral_source_chop.wav",
    "stems/02_w30_feral_source_chop.metrics.md",
    "stems/03_mc202_bass_pressure.wav",
    "stems/03_mc202_bass_pressure.metrics.md",
    "stems/product/01_stem_drums.wav",
    "stems/product/01_stem_drums.metrics.md",
    "stems/product/02_stem_music.wav",
    "stems/product/02_stem_music.metrics.md",
    "stems/product/03_stem_bass.wav",
    "stems/product/03_stem_bass.metrics.md",
    "04_riotbox_source_first_mix.wav",
    "04_riotbox_source_first_mix.metrics.md",
    "05_riotbox_generated_support_mix.wav",
    "05_riotbox_generated_support_mix.metrics.md",
    "grid-report.md",
    "manifest.json",
    "README.md",
];

fn synthetic_source(path: &Path) {
    let samples: Vec<_> = (0..44_100 * 4)
        .flat_map(|frame| {
            let time = frame as f32 / 44_100.0;
            let beat_phase = (time * 128.0 / 60.0).fract();
            let kick =
                (std::f32::consts::TAU * 55.0 * time).sin() * (-beat_phase * 36.0).exp() * 0.5;
            let body = (std::f32::consts::TAU * 82.0 * time).sin() * 0.2;
            [kick + body, (kick + body) * 0.9]
        })
        .collect();
    write_interleaved_pcm16_wav(path, 44_100, 2, &samples).unwrap();
}

fn render(source: &Path, output: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_feral_grid_pack"))
        .arg("--source")
        .arg(source)
        .arg("--output-dir")
        .arg(output)
        .args([
            "--bpm",
            "140",
            "--bars",
            "2",
            "--source-window-seconds",
            "0.5",
        ])
        .output()
        .expect("actual synthetic-only CLI invocation")
}

fn prior_artifacts(output: &Path, skip: &str) {
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if artifact != skip {
            fs::write(path, b"prior artifact bytes").unwrap();
        }
    }
}

fn assert_rejected_without_replacement(source: &Path, output: &Path) {
    let source_before = fs::read(source).unwrap();
    let before: Vec<_> = ARTIFACTS
        .iter()
        .map(|artifact| fs::read(output.join(artifact)).unwrap())
        .collect();
    let result = render(source, output);
    assert!(fs::read(source).unwrap() == source_before, "input changed");
    for (artifact, original) in ARTIFACTS.iter().zip(before) {
        assert!(
            fs::read(output.join(artifact)).unwrap() == original,
            "replaced {artifact}"
        );
    }
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("output aliases input source"),
        "{result:?}"
    );
}

#[test]
fn every_hardlinked_output_pair_is_rejected_without_replacing_any_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    for (index, first) in ARTIFACTS.iter().enumerate() {
        for second in &ARTIFACTS[index + 1..] {
            let output = temp.path().join(format!("pair-{index}-{second}"));
            prior_artifacts(&output, second);
            fs::hard_link(output.join(first), output.join(second)).unwrap();
            assert_rejected_without_replacement(&source, &output);
        }
    }
}

#[cfg(unix)]
#[test]
fn symlinked_audio_metrics_and_metadata_roles_reject_in_both_directions() {
    use std::os::unix::fs::symlink;
    for (first, second) in [
        (ARTIFACTS[0], ARTIFACTS[2]),
        (ARTIFACTS[0], ARTIFACTS[1]),
        (ARTIFACTS[17], ARTIFACTS[18]),
    ] {
        for (target, link) in [(first, second), (second, first)] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("synthetic-control.wav");
            synthetic_source(&source);
            let output = temp.path().join("pack");
            prior_artifacts(&output, link);
            symlink(output.join(target), output.join(link)).unwrap();
            assert_rejected_without_replacement(&source, &output);
        }
    }
}

#[cfg(unix)]
#[test]
fn relative_stem_symlink_under_an_aliased_output_directory_rejects() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let output = temp.path().join("pack");
    prior_artifacts(&output, ARTIFACTS[2]);
    symlink("01_tr909_beat_fill.wav", output.join(ARTIFACTS[2])).unwrap();
    let directory_alias = temp.path().join("directory-alias");
    symlink(&output, &directory_alias).unwrap();
    assert_rejected_without_replacement(&source, &directory_alias);
}

#[test]
fn a_late_metadata_pair_cannot_publish_any_earlier_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let source_before = fs::read(&source).unwrap();
    let output = temp.path().join("pack");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("manifest.json"), b"previous manifest").unwrap();
    fs::hard_link(output.join("manifest.json"), output.join("README.md")).unwrap();
    let result = render(&source, &output);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
    assert!(fs::read(&source).unwrap() == source_before);
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        if ["manifest.json", "README.md"].contains(&artifact) {
            assert_eq!(fs::read(path).unwrap(), b"previous manifest");
        } else {
            assert!(!path.exists(), "published {artifact}");
        }
    }
}

#[test]
fn independent_equal_content_outputs_render_distinct_decodable_stem_roles() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let source_before = fs::read(&source).unwrap();
    let output = temp.path().join("pack");
    prior_artifacts(&output, "no skipped artifact");
    let result = render(&source, &output);
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == source_before);
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        assert!(path.is_file(), "missing {artifact}");
        if artifact.ends_with(".wav") {
            assert!(
                SourceAudioCache::load_pcm_wav(&path).is_ok(),
                "invalid {artifact}"
            );
        }
    }
    assert!(
        fs::read(output.join(ARTIFACTS[0])).unwrap()
            != fs::read(output.join(ARTIFACTS[2])).unwrap()
    );
}
