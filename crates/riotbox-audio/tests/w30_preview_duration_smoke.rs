use std::fs;
use std::process::Command;

#[test]
fn non_finite_duration_fails_before_source_or_artifact_io() {
    for duration in [
        "NaN",
        "nan",
        "+NaN",
        "-NaN",
        "inf",
        "+inf",
        "-inf",
        "infinity",
        "+infinity",
        "-infinity",
    ] {
        for source_argument in [false, true] {
            for existing_artifacts in [false, true] {
                let temp = tempfile::tempdir().expect("tempdir");
                let output_path = temp.path().join("output/preview.wav");
                let metrics_path = temp.path().join("output/preview.metrics.md");
                if existing_artifacts {
                    fs::create_dir(output_path.parent().expect("output parent"))
                        .expect("previous output directory");
                    fs::write(&output_path, b"previous WAV bytes").expect("previous WAV");
                    fs::write(&metrics_path, b"previous metrics bytes").expect("previous metrics");
                }

                let mut command = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"));
                if source_argument {
                    command
                        .arg("--source")
                        .arg(temp.path().join("must-not-be-opened.wav"));
                }
                let result = command
                    .arg("--duration-seconds")
                    .arg(duration)
                    .arg("--out")
                    .arg(&output_path)
                    .output()
                    .expect("run W-30 helper");
                assert_eq!(result.status.code(), Some(1), "{duration}: {result:?}");
                assert!(result.stdout.is_empty(), "{duration}: {result:?}");
                let error = String::from_utf8(result.stderr).expect("UTF-8 error");
                assert!(
                    error.contains("--duration-seconds must be greater than zero"),
                    "{duration}: {error}"
                );
                assert!(!error.contains("panicked"), "{duration}: {error}");

                if existing_artifacts {
                    assert_eq!(
                        fs::read(&output_path).expect("previous WAV"),
                        b"previous WAV bytes"
                    );
                    assert_eq!(
                        fs::read(&metrics_path).expect("previous metrics"),
                        b"previous metrics bytes"
                    );
                } else {
                    assert!(!output_path.exists(), "{duration}: WAV created");
                    assert!(!metrics_path.exists(), "{duration}: metrics created");
                    assert!(
                        !output_path.parent().expect("parent").exists(),
                        "{duration}: output directory created"
                    );
                }
            }
        }
    }
}

#[test]
fn finite_positive_duration_still_renders_the_explicit_synthetic_control() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_path = temp.path().join("preview.wav");
    let metrics_path = temp.path().join("preview.metrics.md");
    let result = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"))
        .arg("--duration-seconds")
        .arg("0.05")
        .arg("--out")
        .arg(&output_path)
        .output()
        .expect("run synthetic control");

    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    assert_eq!(fs::metadata(&output_path).expect("WAV").len(), 8_864);
    let metrics = fs::read_to_string(metrics_path).expect("metrics");
    assert!(metrics.contains("- Source input: `synthetic`"));
    assert!(metrics.contains("- Duration seconds: `0.050`"));
    assert!(metrics.contains("- Samples: `4410`"));
}
