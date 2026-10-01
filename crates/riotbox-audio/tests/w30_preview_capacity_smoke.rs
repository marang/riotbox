use std::{fs, process::Command};

#[test]
fn invalid_render_capacities_fail_before_source_and_artifact_io() {
    // Added only after pure Debug/Release preflight rejection was verified.
    let duration_for_frames = |frames: f64| ((frames / 44_100.0) as f32).to_string();
    let address_limit = isize::MAX as f64 / 2.0 / size_of::<f32>() as f64;
    let buffer_error = "W-30 render buffer capacity exceeded";
    let wav_error = if usize::BITS == 64 {
        "WAV output too large"
    } else {
        buffer_error
    };
    let cases = [
        ("3.4028235e38".to_string(), buffer_error),
        (duration_for_frames(usize::MAX as f64 * 0.75), buffer_error),
        (duration_for_frames(usize::MAX as f64 * 0.25), buffer_error),
        (duration_for_frames(address_limit * 1.5), buffer_error),
        ("30000".to_string(), wav_error),
    ];
    for (duration, expected_error) in cases {
        for source_argument in [false, true] {
            for existing in [false, true] {
                let temp = tempfile::tempdir().unwrap();
                let output = temp.path().join("output/preview.wav");
                let metrics = temp.path().join("output/preview.metrics.md");
                if existing {
                    fs::create_dir(output.parent().unwrap()).unwrap();
                    fs::write(&output, b"prior WAV bytes").unwrap();
                    fs::write(&metrics, b"prior metrics bytes").unwrap();
                }
                let mut command = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"));
                if source_argument {
                    command
                        .arg("--source")
                        .arg(temp.path().join("must-not-be-opened.wav"));
                }
                let rejected = command
                    .arg("--duration-seconds")
                    .arg(&duration)
                    .arg("--out")
                    .arg(&output)
                    .output()
                    .unwrap();
                assert_eq!(rejected.status.code(), Some(1), "{duration}: {rejected:?}");
                assert!(rejected.stdout.is_empty(), "{duration}: {rejected:?}");
                assert_eq!(
                    rejected.stderr,
                    format!("Error: \"{expected_error}\"\n").as_bytes()
                );
                if existing {
                    assert_eq!(fs::read(&output).unwrap(), b"prior WAV bytes");
                    assert_eq!(fs::read(&metrics).unwrap(), b"prior metrics bytes");
                } else {
                    assert!(!output.exists());
                    assert!(!metrics.exists());
                    assert!(!output.parent().unwrap().exists());
                }
            }
        }
    }
}

#[test]
fn help_keeps_precedence_over_finite_capacity_and_source_hydration() {
    let temp = tempfile::tempdir().unwrap();
    let plain = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(plain.status.success());
    assert!(plain.stderr.is_empty());
    let output = temp.path().join("must-not-write/preview.wav");
    let help = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"))
        .args(["--help", "--duration-seconds", "3.4028235e38"])
        .arg("--source")
        .arg(temp.path().join("must-not-be-opened.wav"))
        .arg("--out")
        .arg(&output)
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    assert!(help.stderr.is_empty(), "{help:?}");
    assert_eq!(help.stdout, plain.stdout);
    assert!(!output.parent().unwrap().exists());
}
