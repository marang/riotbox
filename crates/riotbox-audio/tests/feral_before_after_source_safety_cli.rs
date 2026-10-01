use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

// Independent public CLI artifact contract, not the renderer's path plan.
const ARTIFACTS: [&str; 14] = [
    "01_source_excerpt.wav",
    "01_source_excerpt.metrics.md",
    "stems/w30_source_chop.wav",
    "stems/w30_source_chop.metrics.md",
    "stems/tr909_fill.wav",
    "stems/tr909_fill.metrics.md",
    "stems/mc202_instigator.wav",
    "stems/mc202_instigator.metrics.md",
    "02_riotbox_feral_changed.wav",
    "02_riotbox_feral_changed.metrics.md",
    "03_before_then_after.wav",
    "comparison.md",
    "README.md",
    "manifest.json",
];

fn synthetic_source(path: &Path) -> Vec<u8> {
    let samples: Vec<_> = (0..44_100 * 4)
        .flat_map(|frame| {
            let sample = (frame as f32 * 82.0 * std::f32::consts::TAU / 44_100.0).sin() * 0.2;
            [sample, sample * 0.9]
        })
        .collect();
    write_interleaved_pcm16_wav(path, 44_100, 2, &samples).unwrap();
    fs::read(path).unwrap()
}

fn render(source: &Path, output: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_feral_before_after_pack"))
        .arg("--source")
        .arg(source)
        .arg("--output-dir")
        .arg(output)
        .args([
            "--duration-seconds",
            "0.1",
            "--source-window-seconds",
            "0.1",
        ])
        .output()
        .unwrap()
}

fn prior_artifacts(output: &Path, skip: Option<&str>) {
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if Some(artifact) != skip {
            fs::write(path, b"previous artifact bytes").unwrap();
        }
    }
}

fn assert_collision_preserves(source: &Path, output: &Path) {
    let source_before = fs::read(source).unwrap();
    let artifacts_before: Vec<_> = ARTIFACTS
        .iter()
        .map(|artifact| fs::read(output.join(artifact)).unwrap())
        .collect();
    let result = render(source, output);
    assert!(
        fs::read(source).unwrap() == source_before,
        "input must remain intact"
    );
    for (artifact, before) in ARTIFACTS.iter().zip(artifacts_before) {
        assert!(
            fs::read(output.join(artifact)).unwrap() == before,
            "replaced {artifact}"
        );
    }
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
}

#[test]
fn source_excerpt_collision_rejects_before_any_input_or_artifact_replacement() {
    for artifact in ARTIFACTS {
        let temp = tempfile::tempdir().unwrap();
        prior_artifacts(temp.path(), None);
        let source = temp.path().join(artifact);
        synthetic_source(&source);
        // WAV decoding is byte-based; suffixes do not make input disposable.
        assert_collision_preserves(&source, temp.path());
    }
}

#[test]
fn hardlinked_audio_metrics_and_metadata_outputs_preserve_the_input() {
    for artifact in [ARTIFACTS[0], ARTIFACTS[1], ARTIFACTS[13]] {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("pack");
        prior_artifacts(&output, Some(artifact));
        let source = temp.path().join("source.wav");
        synthetic_source(&source);
        fs::hard_link(&source, output.join(artifact)).expect("fixture filesystem hardlink");
        assert_collision_preserves(&source, &output);
    }
}

#[cfg(unix)]
#[test]
fn symlinked_input_output_and_directories_preserve_the_input() {
    use std::os::unix::fs::symlink;
    for artifact in [ARTIFACTS[0], ARTIFACTS[1], ARTIFACTS[13]] {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("pack");
        prior_artifacts(&output, Some(artifact));
        let source = temp.path().join("source.wav");
        synthetic_source(&source);
        symlink(&source, output.join(artifact)).unwrap();
        assert_collision_preserves(&source, &output);
    }
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("pack");
    prior_artifacts(&output, None);
    let source = output.join(ARTIFACTS[0]);
    synthetic_source(&source);
    let input_alias = temp.path().join("input-alias.wav");
    let directory_alias = temp.path().join("directory-alias");
    symlink(&source, &input_alias).unwrap();
    symlink(&output, &directory_alias).unwrap();
    assert_collision_preserves(&input_alias, &directory_alias);
}

#[test]
fn a_late_manifest_collision_does_not_publish_earlier_audio() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("manifest.json");
    let before = synthetic_source(&source);
    let result = render(&source, temp.path());
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
    assert!(fs::read(source).unwrap() == before);
    for artifact in ARTIFACTS
        .into_iter()
        .filter(|path| *path != "manifest.json")
    {
        assert!(!temp.path().join(artifact).exists(), "published {artifact}");
    }
}

#[test]
fn unknown_late_destinations_reject_before_prior_artifacts_are_replaced() {
    for dangling in [false, true] {
        #[cfg(not(unix))]
        if dangling {
            continue;
        }
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("pack");
        prior_artifacts(&output, Some("manifest.json"));
        let source = temp.path().join("source.wav");
        let before = synthetic_source(&source);
        let missing = temp.path().join("absent-target");
        if dangling {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&missing, output.join("manifest.json")).unwrap();
        } else {
            fs::create_dir(output.join("manifest.json")).unwrap();
        }
        let result = render(&source, &output);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
        assert!(fs::read(&source).unwrap() == before);
        assert!(!missing.exists());
        if !dangling {
            assert!(output.join("manifest.json").is_dir());
        }
        for artifact in ARTIFACTS
            .into_iter()
            .filter(|path| *path != "manifest.json")
        {
            assert_eq!(
                fs::read(output.join(artifact)).unwrap(),
                b"previous artifact bytes"
            );
        }
    }
}

#[test]
fn unrelated_input_inside_output_and_equal_content_files_remain_supported() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.wav");
    let before = synthetic_source(&source);
    prior_artifacts(temp.path(), None);
    fs::write(temp.path().join(ARTIFACTS[0]), &before).unwrap();
    let result = render(&source, temp.path());
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == before);
    assert!(fs::read(temp.path().join(ARTIFACTS[0])).unwrap() != before);
    for artifact in ARTIFACTS {
        assert!(temp.path().join(artifact).is_file());
    }
}
