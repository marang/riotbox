use std::{fs, path::Path, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

fn synthetic_source(path: &Path) -> Vec<u8> {
    let samples: Vec<_> = (0..44_100)
        .flat_map(|frame| {
            let sample = (frame as f32 * 110.0 * std::f32::consts::TAU / 44_100.0).sin() * 0.2;
            [sample, sample * 0.9]
        })
        .collect();
    write_interleaved_pcm16_wav(path, 44_100, 2, &samples).unwrap();
    fs::read(path).unwrap()
}

fn render(source: Option<&Path>, output: &Path) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"));
    command
        .arg("--out")
        .arg(output)
        .args(["--duration-seconds", "0.1"]);
    if let Some(source) = source {
        command
            .arg("--source")
            .arg(source)
            .args(["--source-duration-seconds", "0.2"]);
    }
    command.output().unwrap()
}

#[test]
fn hardlinked_wav_and_metrics_reject_without_replacing_either_output() {
    for explicit_source in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.wav");
        let input_before = synthetic_source(&source);
        let output = temp.path().join("preview.wav");
        let metrics = temp.path().join("preview.metrics.md");
        fs::write(&output, b"previous artifact bytes").unwrap();
        fs::hard_link(&output, &metrics).expect("fixture filesystem hardlink");
        let result = render(explicit_source.then_some(source.as_path()), &output);
        assert!(
            fs::read(&output).unwrap() == b"previous artifact bytes",
            "WAV must remain intact"
        );
        assert!(
            fs::read(&metrics).unwrap() == b"previous artifact bytes",
            "metrics must remain intact"
        );
        assert!(fs::read(&source).unwrap() == input_before);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
        assert!(!result.stderr.is_empty(), "{result:?}");
    }
}

#[cfg(unix)]
#[test]
fn symlinked_outputs_cannot_replace_each_other() {
    use std::os::unix::fs::symlink;
    for explicit_source in [false, true] {
        for metrics_alias in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source.wav");
            let before = synthetic_source(&source);
            let output = temp.path().join("preview.wav");
            let metrics = temp.path().join("preview.metrics.md");
            let (alias, target) = if metrics_alias {
                (&metrics, &output)
            } else {
                (&output, &metrics)
            };
            fs::write(target, b"previous artifact bytes").unwrap();
            symlink(target, alias).unwrap();
            let directory_alias = temp.path().join("directory-alias");
            symlink(temp.path(), &directory_alias).unwrap();
            let result = render(
                explicit_source.then_some(source.as_path()),
                &directory_alias.join("preview.wav"),
            );
            assert_eq!(result.status.code(), Some(1), "{result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
            assert!(fs::read(&source).unwrap() == before);
            assert!(fs::read(&output).unwrap() == b"previous artifact bytes");
            assert!(fs::read(&metrics).unwrap() == b"previous artifact bytes");
            assert!(
                fs::symlink_metadata(alias)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
    }
}

#[test]
fn nonregular_outputs_reject_without_replacing_or_publishing_other_output() {
    for explicit_source in [false, true] {
        for metrics_problem in [false, true] {
            for other_missing in [false, true] {
                let temp = tempfile::tempdir().unwrap();
                let source = temp.path().join("source.wav");
                let before = synthetic_source(&source);
                let output = temp.path().join("preview.wav");
                let metrics = temp.path().join("preview.metrics.md");
                let (problem, other) = if metrics_problem {
                    (&metrics, &output)
                } else {
                    (&output, &metrics)
                };
                fs::create_dir(problem).unwrap();
                if !other_missing {
                    fs::write(other, b"previous artifact bytes").unwrap();
                }
                let result = render(explicit_source.then_some(source.as_path()), &output);
                assert_eq!(result.status.code(), Some(1), "{result:?}");
                assert!(result.stdout.is_empty(), "{result:?}");
                assert!(
                    String::from_utf8_lossy(&result.stderr)
                        .contains("output is not a regular file")
                );
                assert!(fs::read(&source).unwrap() == before);
                assert!(problem.is_dir());
                if other_missing {
                    assert!(!other.exists());
                } else {
                    assert_eq!(fs::read(other).unwrap(), b"previous artifact bytes");
                }
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn dangling_outputs_reject_without_target_creation_or_other_replacement() {
    use std::os::unix::fs::symlink;
    for explicit_source in [false, true] {
        for metrics_problem in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source.wav");
            let before = synthetic_source(&source);
            let output = temp.path().join("preview.wav");
            let metrics = temp.path().join("preview.metrics.md");
            let (problem, other) = if metrics_problem {
                (&metrics, &output)
            } else {
                (&output, &metrics)
            };
            let missing = temp.path().join("absent-target");
            fs::write(other, b"previous artifact bytes").unwrap();
            symlink(&missing, problem).unwrap();
            let result = render(explicit_source.then_some(source.as_path()), &output);
            assert_eq!(result.status.code(), Some(1), "{result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
            assert!(fs::read(&source).unwrap() == before);
            assert_eq!(fs::read(other).unwrap(), b"previous artifact bytes");
            assert!(
                fs::symlink_metadata(problem)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert!(!missing.exists());
        }
    }
}

#[test]
fn equal_content_distinct_outputs_still_render_valid_audio_and_metrics() {
    for explicit_source in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.wav");
        let before = synthetic_source(&source);
        let output = temp.path().join("preview.wav");
        let metrics = temp.path().join("preview.metrics.md");
        fs::write(&output, &before).unwrap();
        fs::write(&metrics, &before).unwrap();
        let result = render(explicit_source.then_some(source.as_path()), &output);
        assert!(result.status.success(), "{result:?}");
        assert!(result.stderr.is_empty(), "{result:?}");
        assert!(fs::read(&source).unwrap() == before);
        let cache = riotbox_audio::source_audio::SourceAudioCache::load_pcm_wav(&output).unwrap();
        assert_eq!(cache.frame_count(), 4410);
        assert_eq!(cache.channel_count, 2);
        assert!(
            fs::read_to_string(metrics)
                .unwrap()
                .starts_with("# W-30 Preview Smoke Metrics")
        );
    }
}

#[test]
fn absent_outputs_remain_supported_in_both_diagnostic_modes() {
    for explicit_source in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.wav");
        let before = synthetic_source(&source);
        let output = temp.path().join("nested/preview.wav");
        let result = render(explicit_source.then_some(source.as_path()), &output);
        assert!(result.status.success(), "{result:?}");
        assert!(result.stderr.is_empty(), "{result:?}");
        assert!(fs::read(&source).unwrap() == before);
        let cache = riotbox_audio::source_audio::SourceAudioCache::load_pcm_wav(&output).unwrap();
        assert_eq!(cache.frame_count(), 4410);
        assert!(temp.path().join("nested/preview.metrics.md").is_file());
    }
}
