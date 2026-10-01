use std::{fs, process::Command};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;

#[test]
fn unrepresentable_grids_fail_without_publishing_or_overwriting_audio() {
    let temp = tempfile::tempdir().expect("temporary synthetic control directory");
    let source = temp.path().join("synthetic-control.wav");
    write_interleaved_pcm16_wav(&source, 44_100, 2, &vec![0.0; 8_820])
        .expect("fresh bounded synthetic source");

    // Pure Grid regressions prove rejection before these actual CLI runs.
    // All controls are finite and positive; no huge allocation is attempted.
    let bpm_for_frames = |frames: f64| (8.0 * 44_100.0 * 60.0 / frames) as f32;
    let frame_limit = isize::MAX as f64 / 2.0 / size_of::<f32>() as f64;
    let bpms = [
        "1e-38".to_string(),
        bpm_for_frames(usize::MAX as f64 * 0.75).to_string(),
        bpm_for_frames(usize::MAX as f64 * 0.25).to_string(),
        bpm_for_frames(frame_limit * 1.5).to_string(),
    ];

    for (index, bpm) in bpms.iter().enumerate() {
        for existing in [false, true] {
            let output_dir = temp.path().join(format!("pack-{index}-{existing}"));
            let manifest = output_dir.join("manifest.json");
            let prior_audio = output_dir.join("04_riotbox_source_first_mix.wav");
            if existing {
                fs::create_dir(&output_dir).expect("existing fixture directory");
                fs::write(&manifest, b"prior manifest bytes").expect("manifest sentinel");
                fs::write(&prior_audio, b"prior artifact bytes").expect("artifact sentinel");
            }
            let rejected = Command::new(env!("CARGO_BIN_EXE_feral_grid_pack"))
                .arg("--source")
                .arg(&source)
                .arg("--output-dir")
                .arg(&output_dir)
                .arg("--bpm")
                .arg(bpm)
                .args(["--bars", "2"])
                .output()
                .expect("guarded synthetic CLI invocation");
            assert_eq!(rejected.status.code(), Some(1), "{bpm}: {rejected:?}");
            assert!(rejected.stdout.is_empty(), "{bpm}: {rejected:?}");
            assert_eq!(
                rejected.stderr, b"Error: \"grid render buffer capacity exceeded\"\n",
                "{bpm}: {rejected:?}"
            );
            if existing {
                assert_eq!(fs::read(&manifest).unwrap(), b"prior manifest bytes");
                assert_eq!(fs::read(&prior_audio).unwrap(), b"prior artifact bytes");
            } else {
                assert!(!manifest.exists());
                assert!(!prior_audio.exists());
            }
            for artifact in [
                "README.md",
                "05_riotbox_generated_support_mix.wav",
                "stems/01_tr909_beat_fill.wav",
                "stems/02_w30_feral_source_chop.wav",
                "stems/03_mc202_bass_pressure.wav",
                "stems/product/01_stem_drums.wav",
                "stems/product/02_stem_music.wav",
                "stems/product/03_stem_bass.wav",
            ] {
                assert!(!output_dir.join(artifact).exists(), "unexpected {artifact}");
            }
        }
    }
}
