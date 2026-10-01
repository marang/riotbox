use std::{fs, path::Path, process::Command};

fn comparison_inputs(directory: &Path) -> [std::path::PathBuf; 4] {
    for role in ["baseline", "candidate"] {
        let output = Command::new(env!("CARGO_BIN_EXE_w30_preview_render"))
            .args(["--duration-seconds", "0.1", "--role", role, "--out"])
            .arg(directory.join(format!("{role}.wav")))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
    [
        directory.join("baseline.metrics.md"),
        directory.join("candidate.metrics.md"),
        directory.join("baseline.wav"),
        directory.join("candidate.wav"),
    ]
}

fn compare(inputs: &[std::path::PathBuf; 4], report: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_w30_preview_compare"))
        .arg("--baseline")
        .arg(&inputs[0])
        .arg("--candidate")
        .arg(&inputs[1])
        .arg("--report")
        .arg(report)
        .output()
        .unwrap()
}

#[test]
fn report_collision_preserves_both_metrics_and_associated_wavs() {
    for input_index in 0..4 {
        let temp = tempfile::tempdir().unwrap();
        let inputs = comparison_inputs(temp.path());
        assert_rejected_unchanged(
            &inputs,
            &inputs[input_index],
            &temp.path().join("manifest.json"),
        );
    }
}

fn assert_rejected_unchanged(inputs: &[std::path::PathBuf; 4], report: &Path, manifest: &Path) {
    let before = inputs.each_ref().map(|path| fs::read(path).unwrap());
    let outputs_before = [report, manifest].map(|path| {
        if path.is_file() {
            Some(fs::read(path).unwrap())
        } else {
            None
        }
    });
    let result = compare(inputs, report);
    for (path, bytes) in inputs.iter().zip(before) {
        assert!(fs::read(path).unwrap() == bytes, "input must remain intact");
    }
    for (path, before) in [report, manifest].into_iter().zip(outputs_before) {
        if let Some(bytes) = before {
            assert!(
                fs::read(path).unwrap() == bytes,
                "prior output must remain intact"
            );
        } else {
            assert!(!path.is_file(), "no early publication on late rejection");
        }
    }
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(result.stdout.is_empty(), "{result:?}");
    assert!(!result.stderr.is_empty(), "{result:?}");
}

#[test]
fn hardlinked_report_or_manifest_cannot_replace_any_input() {
    for input_index in 0..4 {
        for manifest_alias in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let inputs = comparison_inputs(temp.path());
            let report = temp.path().join("comparison.md");
            let manifest = temp.path().join("manifest.json");
            let (alias, other) = if manifest_alias {
                (&manifest, &report)
            } else {
                (&report, &manifest)
            };
            fs::write(other, b"previous artifact bytes").unwrap();
            fs::hard_link(&inputs[input_index], alias).expect("fixture filesystem hardlink");
            assert_rejected_unchanged(&inputs, &report, &manifest);
        }
    }
}

#[cfg(unix)]
#[test]
fn output_input_and_directory_symlinks_cannot_replace_inputs() {
    use std::os::unix::fs::symlink;
    for input_index in 0..4 {
        for manifest_alias in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let inputs = comparison_inputs(temp.path());
            let report = temp.path().join("comparison.md");
            let manifest = temp.path().join("manifest.json");
            let (alias, other) = if manifest_alias {
                (&manifest, &report)
            } else {
                (&report, &manifest)
            };
            fs::write(other, b"previous artifact bytes").unwrap();
            symlink(&inputs[input_index], alias).unwrap();
            assert_rejected_unchanged(&inputs, &report, &manifest);
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("pack");
    fs::create_dir(&real).unwrap();
    let inputs = comparison_inputs(&real);
    let directory_alias = temp.path().join("directory-alias");
    symlink(&real, &directory_alias).unwrap();
    let input_alias = temp.path().join("baseline-alias.metrics.md");
    symlink(&inputs[0], &input_alias).unwrap();
    let mut aliased_inputs = inputs;
    aliased_inputs[0] = input_alias;
    // The inferred WAV belongs to the metrics-path convention, not its target.
    fs::copy(&aliased_inputs[2], temp.path().join("baseline-alias.wav")).unwrap();
    assert_rejected_unchanged(
        &aliased_inputs,
        &directory_alias.join("baseline.metrics.md"),
        &real.join("manifest.json"),
    );
}

#[test]
fn late_manifest_collision_does_not_publish_an_absent_report() {
    for input_index in 0..4 {
        let temp = tempfile::tempdir().unwrap();
        let inputs = comparison_inputs(temp.path());
        let report = temp.path().join("comparison.md");
        let manifest = temp.path().join("manifest.json");
        fs::hard_link(&inputs[input_index], &manifest).expect("fixture filesystem hardlink");
        assert_rejected_unchanged(&inputs, &report, &manifest);
        assert!(!report.exists());
    }
}

#[test]
fn nonregular_or_dangling_outputs_reject_before_any_replacement() {
    for manifest_problem in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let inputs = comparison_inputs(temp.path());
        let report = temp.path().join("comparison.md");
        let manifest = temp.path().join("manifest.json");
        let (problem, other) = if manifest_problem {
            (&manifest, &report)
        } else {
            (&report, &manifest)
        };
        fs::write(other, b"previous artifact bytes").unwrap();
        fs::create_dir(problem).unwrap();
        assert_rejected_unchanged(&inputs, &report, &manifest);
        assert!(problem.is_dir());
        #[cfg(unix)]
        {
            fs::remove_dir(problem).unwrap();
            let absent = temp.path().join("absent-target");
            std::os::unix::fs::symlink(&absent, problem).unwrap();
            assert_rejected_unchanged(&inputs, &report, &manifest);
            assert!(
                fs::symlink_metadata(problem)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert!(!absent.exists());
        }
    }
}

#[test]
fn missing_or_nonregular_associated_audio_cannot_publish_outputs() {
    for input_index in [2, 3] {
        for directory_input in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let inputs = comparison_inputs(temp.path());
            fs::remove_file(&inputs[input_index]).unwrap();
            if directory_input {
                fs::create_dir(&inputs[input_index]).unwrap();
            }
            let report = temp.path().join("comparison.md");
            let result = compare(&inputs, &report);
            assert!(
                !report.exists(),
                "invalid associated input must reject before publication"
            );
            assert!(!temp.path().join("manifest.json").exists());
            assert_eq!(result.status.code(), Some(1), "{result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn independent_equal_content_outputs_and_shared_metrics_input_remain_supported() {
    let temp = tempfile::tempdir().unwrap();
    let mut inputs = comparison_inputs(temp.path());
    inputs[1] = inputs[0].clone();
    let before = inputs.each_ref().map(|path| fs::read(path).unwrap());
    let report = temp.path().join("comparison.md");
    let manifest = temp.path().join("manifest.json");
    fs::write(&report, &before[0]).unwrap();
    fs::write(&manifest, &before[0]).unwrap();
    let result = compare(&inputs, &report);
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    for (path, bytes) in inputs.iter().zip(before) {
        assert!(fs::read(path).unwrap() == bytes);
    }
    assert!(
        fs::read_to_string(&report)
            .unwrap()
            .starts_with("W-30 preview smoke metrics comparison")
    );
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    assert_eq!(json["result"], "pass");
}

#[test]
fn report_and_manifest_cannot_replace_each_other() {
    for existing_hardlink in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let inputs = comparison_inputs(temp.path());
        let before = inputs.each_ref().map(|path| fs::read(path).unwrap());
        let manifest = temp.path().join("manifest.json");
        let report = if existing_hardlink {
            let report = temp.path().join("comparison.md");
            fs::write(&report, b"previous report bytes").unwrap();
            fs::hard_link(&report, &manifest).expect("fixture filesystem hardlink");
            report
        } else {
            manifest.clone()
        };
        let result = compare(&inputs, &report);
        for (path, bytes) in inputs.iter().zip(before) {
            assert!(fs::read(path).unwrap() == bytes);
        }
        if existing_hardlink {
            assert_eq!(fs::read(&report).unwrap(), b"previous report bytes");
            assert_eq!(fs::read(&manifest).unwrap(), b"previous report bytes");
        } else {
            assert!(!report.exists(), "colliding outputs must not be published");
        }
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
    }
}
