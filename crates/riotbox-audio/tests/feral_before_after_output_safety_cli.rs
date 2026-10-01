use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::{SourceAudioCache, write_interleaved_pcm16_wav};

// Independent literal CLI contract; the composite has no metrics file.
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

fn synthetic_source(path: &Path) {
    let samples: Vec<_> = (0..44_100 * 4)
        .flat_map(|frame| {
            let sample = (frame as f32 * 82.0 * std::f32::consts::TAU / 44_100.0).sin() * 0.2;
            [sample, sample * 0.9]
        })
        .collect();
    write_interleaved_pcm16_wav(path, 44_100, 2, &samples).unwrap();
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
        .expect("actual synthetic-only CLI invocation")
}

fn prior_artifacts(output: &Path, skip: &str) {
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if artifact != skip {
            fs::write(path, b"previous artifact bytes").unwrap();
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
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
}

#[test]
fn every_hardlinked_output_pair_rejects_before_any_artifact_is_replaced() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    for (index, first) in ARTIFACTS.iter().enumerate() {
        for (offset, second) in ARTIFACTS[index + 1..].iter().enumerate() {
            let output = temp
                .path()
                .join(format!("pair-{index}-{}", index + offset + 1));
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
        (ARTIFACTS[2], ARTIFACTS[4]),
        (ARTIFACTS[0], ARTIFACTS[1]),
        (ARTIFACTS[12], ARTIFACTS[13]),
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
fn relative_stem_link_through_an_aliased_output_directory_rejects() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let output = temp.path().join("pack");
    prior_artifacts(&output, ARTIFACTS[4]);
    symlink("w30_source_chop.wav", output.join(ARTIFACTS[4])).unwrap();
    let directory_alias = temp.path().join("directory-alias");
    symlink(&output, &directory_alias).unwrap();
    assert_rejected_without_replacement(&source, &directory_alias);
}

#[test]
fn late_readme_manifest_coupling_cannot_publish_the_source_excerpt_or_other_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let before = fs::read(&source).unwrap();
    let output = temp.path().join("pack");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("README.md"), b"previous metadata").unwrap();
    fs::hard_link(output.join("README.md"), output.join("manifest.json")).unwrap();
    let result = render(&source, &output);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
    assert!(fs::read(&source).unwrap() == before);
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        if ["README.md", "manifest.json"].contains(&artifact) {
            assert_eq!(fs::read(path).unwrap(), b"previous metadata");
        } else {
            assert!(!path.exists(), "published {artifact}");
        }
    }
}

#[test]
fn independent_equal_content_outputs_keep_distinct_decodable_stems() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    synthetic_source(&source);
    let before = fs::read(&source).unwrap();
    let output = temp.path().join("pack");
    prior_artifacts(&output, "no skipped artifact");
    let result = render(&source, &output);
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == before);
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        assert!(path.is_file(), "missing {artifact}");
        if artifact.ends_with(".wav") {
            let cache = SourceAudioCache::load_pcm_wav(&path).unwrap();
            assert_eq!(cache.channel_count, 2);
            assert_eq!(
                cache.frame_count(),
                if artifact == "03_before_then_after.wav" {
                    // 0.1s before + the existing 0.75s gap + 0.1s after.
                    41895
                } else {
                    4410
                }
            );
        }
    }
    assert!(
        fs::read(output.join(ARTIFACTS[2])).unwrap()
            != fs::read(output.join(ARTIFACTS[4])).unwrap()
    );
}
