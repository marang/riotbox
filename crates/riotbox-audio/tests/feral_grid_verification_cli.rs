#![cfg(unix)]

use std::{fs, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

#[test]
fn synthetic_cli_manifest_keeps_literal_reproduction_arguments() {
    let temp = tempfile::tempdir().expect("temporary synthetic control directory");
    let source = temp
        .path()
        .join("control '\" $RBX_QUOTE_PROBE $(printf expanded).wav");
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
    write_interleaved_pcm16_wav(&source, 44_100, 2, &samples).expect("fresh synthetic control WAV");
    let date = "review '\" $RBX_QUOTE_PROBE $(printf expanded) 🎛";

    for explicit in [false, true] {
        let output_dir = temp.path().join(if explicit { "explicit" } else { "auto" });
        let mut command = Command::new(env!("CARGO_BIN_EXE_feral_grid_pack"));
        command
            .arg("--source")
            .arg(&source)
            .arg("--date")
            .arg(date)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--bars")
            .arg("2")
            .arg("--source-window-seconds")
            .arg("0.5");
        if explicit {
            command.arg("--bpm").arg("140");
        }
        let rendered = command.output().expect("exact synthetic CLI render");
        assert!(rendered.status.success(), "{rendered:?}");
        assert!(rendered.stderr.is_empty(), "{rendered:?}");
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(output_dir.join("manifest.json")).expect("manifest bytes"),
        )
        .expect("manifest JSON");
        let verification = manifest["verification_command"]
            .as_str()
            .expect("verification command");
        // Evaluate only against an argv printer, never real Just/Cargo or audio playback.
        let probe = Command::new("sh")
            .args([
                "-c",
                &("just() { printf '%s\\000' \"$@\"; }\n".to_string() + verification),
            ])
            .env("RBX_QUOTE_PROBE", "expanded")
            .env_remove("ENV")
            .env_remove("BASH_ENV")
            .output()
            .expect("manifest command argv-only probe");
        assert!(probe.status.success(), "{probe:?}");
        assert!(probe.stderr.is_empty(), "{probe:?}");
        let source_text = source.to_str().expect("UTF-8 control path");
        let bpm = if explicit { "140.000" } else { "auto" };
        let expected: Vec<_> = [
            "feral-grid-pack",
            source_text,
            date,
            bpm,
            "2",
            "0.500",
            "0.000",
            "",
        ]
        .into_iter()
        .map(str::as_bytes)
        .collect();
        assert_eq!(
            probe.stdout.split(|byte| *byte == 0).collect::<Vec<_>>(),
            expected
        );
        let readme = fs::read_to_string(output_dir.join("README.md")).expect("README");
        assert!(readme.contains(&format!("- Source: `{source_text}`")));
        assert!(readme.contains("This is an offline QA/listening pack."));
    }
}
