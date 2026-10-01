use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

// Independent CLI artifact contract, not the production path planner.
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

fn synthetic_source(path: &Path) -> Vec<u8> {
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
    write_interleaved_pcm16_wav(path, 44_100, 2, &samples).expect("fresh synthetic control");
    fs::read(path).expect("original synthetic source bytes")
}

fn render(source: &Path, output: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_feral_grid_pack"))
        .arg("--source")
        .arg(source)
        .arg("--output-dir")
        .arg(output)
        .args(["--bars", "2", "--source-window-seconds", "0.5"])
        .output()
        .expect("actual synthetic-only CLI invocation")
}

fn prior_artifacts(output: &Path, skip: Option<&str>) {
    for artifact in ARTIFACTS {
        let path = output.join(artifact);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if Some(artifact) != skip {
            fs::write(path, b"prior artifact bytes").unwrap();
        }
    }
}

fn assert_collision_preserves_all(source: &Path, output: &Path) {
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
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("output aliases input source"),
        "{result:?}"
    );
}

#[test]
fn colliding_source_is_rejected_without_replacing_source_or_prior_artifacts() {
    for artifact in ARTIFACTS {
        let temp = tempfile::tempdir().expect("owned synthetic directory");
        prior_artifacts(temp.path(), None);
        let source = temp.path().join(artifact);
        synthetic_source(&source);
        // WAV decoding is byte-based; metadata suffixes are not protection.
        assert_collision_preserves_all(&source, temp.path());
    }
}

#[test]
fn hardlinked_outputs_are_rejected_before_any_artifact_is_replaced() {
    for artifact in [ARTIFACTS[0], ARTIFACTS[1], ARTIFACTS[18]] {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("pack");
        prior_artifacts(&output, Some(artifact));
        let source = temp.path().join("synthetic-control.wav");
        synthetic_source(&source);
        fs::hard_link(&source, output.join(artifact)).expect("fixture filesystem hardlink");
        assert_collision_preserves_all(&source, &output);
    }
}

#[test]
fn a_late_metadata_collision_does_not_publish_earlier_audio() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("README.md");
    let before = synthetic_source(&source);
    let result = render(&source, temp.path());
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
    assert!(fs::read(&source).unwrap() == before);
    for artifact in ARTIFACTS.into_iter().filter(|path| *path != "README.md") {
        assert!(!temp.path().join(artifact).exists(), "published {artifact}");
    }
}

#[cfg(unix)]
#[test]
fn symlinked_outputs_and_directory_aliases_cannot_replace_the_source() {
    use std::os::unix::fs::symlink;
    for artifact in [ARTIFACTS[0], ARTIFACTS[1], ARTIFACTS[18]] {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("pack");
        prior_artifacts(&output, Some(artifact));
        let source = temp.path().join("synthetic-control.wav");
        synthetic_source(&source);
        symlink(&source, output.join(artifact)).unwrap();
        assert_collision_preserves_all(&source, &output);
    }
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("pack");
    prior_artifacts(&output, None);
    let source = output.join(ARTIFACTS[0]);
    synthetic_source(&source);
    let directory_alias = temp.path().join("directory-alias");
    symlink(&output, &directory_alias).unwrap();
    let input_alias = temp.path().join("input-alias.wav");
    symlink(&source, &input_alias).unwrap();
    assert_collision_preserves_all(&input_alias, &directory_alias);
}

#[cfg(unix)]
#[test]
fn dangling_output_symlink_fails_closed_without_replacing_prior_audio() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("pack");
    prior_artifacts(&output, Some("README.md"));
    let missing = temp.path().join("absent-target");
    symlink(&missing, output.join("README.md")).unwrap();
    let source = temp.path().join("synthetic-control.wav");
    let before = synthetic_source(&source);
    let result = render(&source, &output);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(!missing.exists());
    assert!(fs::read(&source).unwrap() == before);
    for artifact in ARTIFACTS.into_iter().filter(|path| *path != "README.md") {
        assert_eq!(
            fs::read(output.join(artifact)).unwrap(),
            b"prior artifact bytes"
        );
    }
}

#[test]
fn unrelated_input_inside_output_directory_remains_a_supported_source() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("synthetic-control.wav");
    let before = synthetic_source(&source);
    // Equal bytes are not a file-identity collision. Ordinary output files
    // may be replaced; the distinct input remains untouched.
    prior_artifacts(temp.path(), None);
    fs::write(temp.path().join(ARTIFACTS[0]), &before).unwrap();
    let result = render(&source, temp.path());
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == before);
    assert!(fs::read(temp.path().join(ARTIFACTS[0])).unwrap() != before);
    for artifact in ARTIFACTS {
        assert!(temp.path().join(artifact).is_file(), "missing {artifact}");
    }
}
