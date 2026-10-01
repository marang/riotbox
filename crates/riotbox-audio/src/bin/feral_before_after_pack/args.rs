use super::config::{
    DEFAULT_DATE, DEFAULT_DURATION_SECONDS, DEFAULT_SOURCE_START_SECONDS,
    DEFAULT_SOURCE_WINDOW_SECONDS, PACK_ID,
};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub(super) struct Args {
    pub(super) source_path: PathBuf,
    pub(super) output_dir: Option<PathBuf>,
    pub(super) date: String,
    pub(super) source_start_seconds: f32,
    pub(super) duration_seconds: f32,
    pub(super) source_window_seconds: f32,
    pub(super) show_help: bool,
}

impl Args {
    pub(super) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source_path = None;
        let mut output_dir = None;
        let mut date = DEFAULT_DATE.to_string();
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
        let mut duration_seconds = DEFAULT_DURATION_SECONDS;
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
                "--source-start-seconds" => {
                    source_start_seconds = parse_non_negative_seconds(
                        "--source-start-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--source-start-seconds requires a value".to_string())?,
                    )?;
                }
                "--duration-seconds" => {
                    duration_seconds = parse_positive_seconds(
                        "--duration-seconds",
                        &args
                            .next()
                            .ok_or_else(|| "--duration-seconds requires a value".to_string())?,
                    )?;
                }
                "--source-window-seconds" => {
                    source_window_seconds = parse_positive_seconds(
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
            source_start_seconds,
            duration_seconds,
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
        "Usage: feral_before_after_pack --source PATH [--date NAME] [--output-dir PATH]\n\
         \n\
         Optional window controls:\n\
           --source-start-seconds SECONDS\n\
           --duration-seconds SECONDS\n\
           --source-window-seconds SECONDS\n\
         \n\
         Renders a local Feral before/after listening pack with source excerpt,\n\
         Riotbox-transformed after render, before-then-after comparison, stems,\n\
         metrics, and README. This helper currently expects 44.1 kHz stereo PCM WAV input."
    );
}

fn parse_non_negative_seconds(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be a non-negative number"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("{flag} must be a non-negative number"));
    }
    Ok(parsed)
}

fn parse_positive_seconds(flag: &str, value: &str) -> Result<f32, String> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("{flag} must be greater than zero"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{flag} must be greater than zero"));
    }
    Ok(parsed)
}
