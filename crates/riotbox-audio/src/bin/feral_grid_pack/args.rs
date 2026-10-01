use std::path::{Path, PathBuf};

use super::config::{
    DEFAULT_BARS, DEFAULT_BPM, DEFAULT_DATE, DEFAULT_SOURCE_START_SECONDS,
    DEFAULT_SOURCE_WINDOW_SECONDS, MIN_BARS, PACK_ID,
};

#[derive(Debug, PartialEq)]
pub(super) struct Args {
    pub(super) source_path: PathBuf,
    pub(super) output_dir: Option<PathBuf>,
    pub(super) date: String,
    pub(super) bpm: f32,
    pub(super) bpm_overridden: bool,
    pub(super) bars: u32,
    pub(super) source_start_seconds: f32,
    pub(super) source_window_seconds: f32,
    pub(super) show_help: bool,
}

impl Args {
    pub(super) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source_path = None;
        let mut output_dir = None;
        let mut date = DEFAULT_DATE.to_string();
        let mut bpm = DEFAULT_BPM;
        let mut bpm_overridden = false;
        let mut bars = DEFAULT_BARS;
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
        let mut source_window_seconds = DEFAULT_SOURCE_WINDOW_SECONDS;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--source" => {
                    source_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--source requires a path".to_string())?,
                    ));
                }
                "--output-dir" => {
                    output_dir =
                        Some(PathBuf::from(args.next().ok_or_else(|| {
                            "--output-dir requires a value".to_string()
                        })?));
                }
                "--date" => {
                    date = args
                        .next()
                        .ok_or_else(|| "--date requires a value".to_string())?;
                }
                "--bpm" => {
                    bpm = parse_positive_f32(
                        "--bpm",
                        &args
                            .next()
                            .ok_or_else(|| "--bpm requires a value".to_string())?,
                    )?;
                    bpm_overridden = true;
                }
                "--bars" => {
                    bars = parse_bars(
                        &args
                            .next()
                            .ok_or_else(|| "--bars requires a value".to_string())?,
                    )?;
                }
                "--source-start-seconds" => {
                    source_start_seconds = parse_non_negative_f32(
                        "--source-start-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--source-start-seconds requires a value".to_string())?,
                    )?;
                }
                "--source-window-seconds" => {
                    source_window_seconds = parse_positive_f32(
                        "--source-window-seconds",
                        &args.next().ok_or_else(|| {
                            "--source-window-seconds requires a value".to_string()
                        })?,
                    )?;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let source_path = source_path.ok_or_else(|| "--source is required".to_string())?;

        Ok(Self {
            source_path,
            output_dir,
            date,
            bpm,
            bpm_overridden,
            bars,
            source_start_seconds,
            source_window_seconds,
            show_help,
        })
    }

    pub(super) fn output_dir(&self) -> PathBuf {
        self.output_dir.clone().unwrap_or_else(|| {
            Path::new("artifacts")
                .join("audio_qa")
                .join(&self.date)
                .join(PACK_ID)
        })
    }
}

pub(super) fn print_help() {
    println!(
        "Usage: feral_grid_pack --source PATH [--date NAME] [--output-dir PATH]\n\
         \n\
         Optional grid controls:\n\
          --bpm BPM              Override source-timing BPM selection\n\
           --bars BARS\n\
           --source-start-seconds SECONDS\n\
           --source-window-seconds SECONDS\n\
         \n\
         Renders a local grid-locked Feral demo pack. Without --bpm, the pack\n\
         uses ready source timing that does not require manual confirmation\n\
         when available and otherwise falls back to\n\
         the static default BPM. TR-909 beat/fill, MC-202 bass pressure,\n\
         and W-30 source chop stems share one beat/bar grid\n\
         so the output can be checked for musical timing instead of only logs."
    );
}

fn parse_positive_f32(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

fn parse_non_negative_f32(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be a non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("{flag} must be a non-negative number"));
    }
    Ok(parsed)
}

fn parse_positive_u32(flag: &str, value: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}

fn parse_bars(value: &str) -> Result<u32, String> {
    let bars = parse_positive_u32("--bars", value)?;
    if bars < MIN_BARS {
        return Err(format!("--bars must be at least {MIN_BARS}"));
    }
    Ok(bars)
}
