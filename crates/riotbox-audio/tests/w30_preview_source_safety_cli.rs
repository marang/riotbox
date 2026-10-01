use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

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
    Command::new(env!("CARGO_BIN_EXE_w30_preview_render"))
        .arg("--source")
        .arg(source)
        .arg("--out")
        .arg(output)
        .args([
            "--duration-seconds",
            "0.1",
            "--source-duration-seconds",
            "0.2",
        ])
        .output()
        .unwrap()
}

fn assert_collision_preserves(source: &Path, output: &Path, metrics: &Path) {
    let before = fs::read(source).unwrap();
    let output_before = fs::read(output).unwrap();
    let metrics_before = fs::read(metrics).unwrap();
    let result = render(source, output);
    assert!(
        fs::read(source).unwrap() == before,
        "input must remain intact"
    );
    assert!(
        fs::read(output).unwrap() == output_before,
        "prior WAV must remain intact"
    );
    assert!(
        fs::read(metrics).unwrap() == metrics_before,
        "prior metrics must remain intact"
    );
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output aliases input source"));
}

#[test]
fn source_collision_rejects_without_replacing_wav_or_metrics() {
    for metrics_source in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("preview.wav");
        let metrics = temp.path().join("preview.metrics.md");
        fs::write(&output, b"previous WAV bytes").unwrap();
        fs::write(&metrics, b"previous metrics bytes").unwrap();
        let source = if metrics_source { &metrics } else { &output };
        synthetic_source(source);
        assert_collision_preserves(source, &output, &metrics);
    }
}

#[test]
fn hardlinked_wav_and_metrics_outputs_cannot_replace_the_input() {
    for metrics_alias in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.wav");
        synthetic_source(&source);
        let output = temp.path().join("preview.wav");
        let metrics = temp.path().join("preview.metrics.md");
        let (alias, other) = if metrics_alias {
            (&metrics, &output)
        } else {
            (&output, &metrics)
        };
        fs::write(other, b"previous artifact bytes").unwrap();
        fs::hard_link(&source, alias).expect("fixture filesystem hardlink");
        assert_collision_preserves(&source, &output, &metrics);
    }
}

#[cfg(unix)]
#[test]
fn symlinks_and_directory_aliases_cannot_replace_the_input() {
    use std::os::unix::fs::symlink;
    for metrics_alias in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.wav");
        synthetic_source(&source);
        let output = temp.path().join("preview.wav");
        let metrics = temp.path().join("preview.metrics.md");
        let (alias, other) = if metrics_alias {
            (&metrics, &output)
        } else {
            (&output, &metrics)
        };
        fs::write(other, b"previous artifact bytes").unwrap();
        symlink(&source, alias).unwrap();
        assert_collision_preserves(&source, &output, &metrics);
    }
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("pack");
    fs::create_dir(&real).unwrap();
    let output = real.join("preview.wav");
    let metrics = real.join("preview.metrics.md");
    synthetic_source(&metrics);
    fs::write(&output, b"previous WAV bytes").unwrap();
    let directory_alias = temp.path().join("directory-alias");
    let input_alias = temp.path().join("input-alias.wav");
    symlink(&real, &directory_alias).unwrap();
    symlink(&metrics, &input_alias).unwrap();
    assert_collision_preserves(&input_alias, &directory_alias.join("preview.wav"), &metrics);
}

#[cfg(unix)]
#[test]
fn dangling_metrics_output_rejects_before_wav_replacement() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.wav");
    let before = synthetic_source(&source);
    let output = temp.path().join("preview.wav");
    let metrics = temp.path().join("preview.metrics.md");
    let missing = temp.path().join("absent-target");
    fs::write(&output, b"previous WAV bytes").unwrap();
    symlink(&missing, &metrics).unwrap();
    let result = render(&source, &output);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == before);
    assert_eq!(fs::read(&output).unwrap(), b"previous WAV bytes");
    assert!(!missing.exists());
}

#[test]
fn nonregular_metrics_destination_rejects_before_wav_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.wav");
    let before = synthetic_source(&source);
    let output = temp.path().join("preview.wav");
    let metrics = temp.path().join("preview.metrics.md");
    fs::write(&output, b"previous WAV bytes").unwrap();
    fs::create_dir(&metrics).unwrap();
    let result = render(&source, &output);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("output is not a regular file"));
    assert!(fs::read(&source).unwrap() == before);
    assert_eq!(fs::read(&output).unwrap(), b"previous WAV bytes");
    assert!(metrics.is_dir());
}

#[test]
fn independent_equal_content_output_and_input_inside_output_remain_supported() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.wav");
    let before = synthetic_source(&source);
    let output = temp.path().join("preview.wav");
    fs::write(&output, &before).unwrap();
    let result = render(&source, &output);
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert!(fs::read(&source).unwrap() == before);
    assert!(fs::read(&output).unwrap() != before);
    assert!(temp.path().join("preview.metrics.md").is_file());
}
