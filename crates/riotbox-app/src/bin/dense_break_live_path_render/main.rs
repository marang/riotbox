mod alpha_arc;
mod alpha_manifest;
mod controlled_source_manifest;
mod live_flow;
mod manifest;
mod model;
mod rendering;
mod tonal_journey;
mod tonal_live_manifest;

use std::{env, fs, path::PathBuf};

const MIN_SOURCE_BPM_HINT: f32 = 20.0;
const MAX_SOURCE_BPM_HINT: f32 = 400.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let source_path = required_path(&args, "--source")?;
    let output_dir = required_path(&args, "--output")?;
    let cli_bpm_hint = required_bpm(&args)?;
    let cli_downbeat_seconds = optional_non_negative_f32(&args, "--downbeat-seconds")?;
    let controlled_source_review = args.iter().any(|arg| arg == "--controlled-source-review");
    let tonal_live_review = args.iter().any(|arg| arg == "--tonal-live-review");
    if controlled_source_review && tonal_live_review {
        return Err("choose only one review mode".into());
    }
    for directory in [
        "stems",
        "monitor",
        "gestures",
        "gestures/proofs",
        "alpha",
        "controlled",
        "tonal",
    ] {
        fs::create_dir_all(output_dir.join(directory))?;
    }

    let prepared = live_flow::prepare(
        &source_path,
        &output_dir,
        cli_bpm_hint,
        cli_downbeat_seconds,
    )?;
    if tonal_live_review {
        tonal_live_manifest::write_pack(prepared, &source_path, &output_dir)
    } else if controlled_source_review {
        let rendered = rendering::render_live_path(&prepared)?;
        controlled_source_manifest::write_pack(prepared, rendered, &source_path, &output_dir)
    } else {
        let rendered = rendering::render_live_path(&prepared)?;
        manifest::write_pack(prepared, rendered, &source_path, &output_dir)
    }
}

fn required_bpm(args: &[String]) -> Result<f32, Box<dyn std::error::Error>> {
    let value = required_value(args, "--bpm")?;
    let bpm = value
        .parse::<f32>()
        .map_err(|_| format!("invalid BPM value: {value}"))?;
    if !bpm.is_finite() || !(MIN_SOURCE_BPM_HINT..=MAX_SOURCE_BPM_HINT).contains(&bpm) {
        return Err(format!(
            "invalid BPM value: {value}; expected a finite source-confirmation hint in {MIN_SOURCE_BPM_HINT}..={MAX_SOURCE_BPM_HINT}"
        )
        .into());
    }
    Ok(bpm)
}

fn required_path(args: &[String], flag: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(required_value(args, flag)?))
}

fn optional_non_negative_f32(
    args: &[String],
    flag: &str,
) -> Result<Option<f32>, Box<dyn std::error::Error>> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };
    let value = args
        .get(index + 1)
        .ok_or_else(|| format!("missing value for {flag}"))?;
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("invalid value for {flag}: {value}").into());
    }
    Ok(Some(parsed))
}

fn required_value<'a>(
    args: &'a [String],
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let index = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or(flag.to_string())?;
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bpm_args(value: &str) -> Vec<String> {
        vec!["--bpm".into(), value.into()]
    }

    #[test]
    fn accepts_finite_bpm_in_diagnostic_render_range() {
        assert_eq!(required_bpm(&bpm_args("132")).unwrap(), 132.0);
        assert_eq!(required_bpm(&bpm_args("20")).unwrap(), 20.0);
        assert_eq!(required_bpm(&bpm_args("400")).unwrap(), 400.0);
    }

    #[test]
    fn rejects_out_of_range_or_non_finite_bpm() {
        for value in ["0", "-1", "19.99", "400.01", "NaN", "inf", "-inf"] {
            let error = required_bpm(&bpm_args(value)).expect_err("BPM must be rejected");
            assert!(
                error
                    .to_string()
                    .contains("expected a finite source-confirmation hint"),
                "unexpected error for {value}: {error}"
            );
        }
    }

    #[test]
    fn parses_optional_manual_downbeat_seconds() {
        let args = vec!["--downbeat-seconds".into(), "0.125".into()];
        assert_eq!(
            optional_non_negative_f32(&args, "--downbeat-seconds").unwrap(),
            Some(0.125)
        );
        assert_eq!(
            optional_non_negative_f32(&[], "--downbeat-seconds").unwrap(),
            None
        );
    }
}
