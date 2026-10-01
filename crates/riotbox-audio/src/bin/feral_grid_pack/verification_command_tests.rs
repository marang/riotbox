use super::{args::Args, grid::Grid, verification_command::verification_command};

fn command_for(source: &str, date: &str, explicit_bpm: bool) -> String {
    let mut argv = vec![
        "--source".to_string(),
        source.to_string(),
        "--date".to_string(),
        date.to_string(),
        "--bars".to_string(),
        "2".to_string(),
        "--source-start-seconds".to_string(),
        "0.125".to_string(),
    ];
    if explicit_bpm {
        argv.extend(["--bpm".to_string(), "120.125".to_string()]);
    }
    let args = Args::parse(argv).expect("valid CLI arguments");
    let grid = Grid::new(120.125, 4, 2).expect("bounded grid");
    verification_command(&args, &grid, 0.75)
}

#[test]
fn verification_command_quotes_source_and_date_with_stable_numeric_controls() {
    assert_eq!(
        command_for("control\"quote.wav", "review's date", false),
        "just feral-grid-pack 'control\"quote.wav' 'review'\\''s date' auto 2 0.750 0.125"
    );
    assert_eq!(
        command_for("control.wav", "", true),
        "just feral-grid-pack 'control.wav' '' 120.125 2 0.750 0.125"
    );
}

#[cfg(unix)]
#[test]
fn verification_command_preserves_literal_posix_argv_without_expansion() {
    use std::process::Command;

    // This stub only emits argv. It never invokes Just, Cargo, audio or an external payload.
    let stub = "just() { printf '%s\\000' \"$@\"; }\n";
    for value in [
        "plain",
        "space tab\tline\nend",
        "single'and\"double",
        "back\\slash",
        "$RBX_QUOTE_PROBE",
        "$(printf expanded)",
        "`printf expanded`",
        "semi;colon",
        "--option-like",
        "Grüße 🎛",
        "",
    ] {
        for explicit in [false, true] {
            let command = command_for(value, value, explicit);
            let output = Command::new("sh")
                .args(["-c", &(stub.to_string() + &command)])
                .env("RBX_QUOTE_PROBE", "expanded")
                .env_remove("ENV")
                .env_remove("BASH_ENV")
                .output()
                .expect("POSIX argv-only probe");
            assert!(
                output.status.success(),
                "value={value:?}: {:?}",
                output.stderr
            );
            assert!(
                output.stderr.is_empty(),
                "value={value:?}: {:?}",
                output.stderr
            );
            let actual: Vec<_> = output.stdout.split(|byte| *byte == 0).collect();
            let bpm = if explicit { "120.125" } else { "auto" };
            let expected: Vec<_> = [
                "feral-grid-pack",
                value,
                value,
                bpm,
                "2",
                "0.750",
                "0.125",
                "",
            ]
            .into_iter()
            .map(str::as_bytes)
            .collect();
            assert_eq!(actual, expected, "value={value:?}, explicit={explicit}");
        }
    }
}
