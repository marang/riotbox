use super::config::DEFAULT_DURATION_SECONDS;
use super::config::PACK_ID;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Args {
    pub(super) date: String,
    pub(super) output_dir: Option<PathBuf>,
    pub(super) duration_seconds: f32,
    pub(super) show_help: bool,
}

pub(super) fn print_help() {
    println!(
        "Usage: lane_recipe_pack [--date NAME] [--output-dir PATH] [--duration-seconds SECONDS]\n\
         \n\
         Renders a local lane-level recipe listening pack with TR-909, MC-202, and\n\
         Scene-coupled support cases. Writes WAV, metrics, comparison reports, and pack-summary.md.\n\
         This is a local QA helper; generated audio artifacts stay under artifacts/audio_qa/."
    );
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

impl Default for Args {
    fn default() -> Self {
        Self {
            date: "local".into(),
            output_dir: None,
            duration_seconds: DEFAULT_DURATION_SECONDS,
            show_help: false,
        }
    }
}

impl Args {
    pub(super) fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => parsed.show_help = true,
                "--date" => {
                    let Some(value) = args.next() else {
                        return Err("--date requires a value".into());
                    };
                    parsed.date = value;
                }
                "--output-dir" => {
                    let Some(value) = args.next() else {
                        return Err("--output-dir requires a value".into());
                    };
                    parsed.output_dir = Some(PathBuf::from(value));
                }
                "--duration-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--duration-seconds requires a value".into());
                    };
                    parsed.duration_seconds = parse_positive_seconds("--duration-seconds", &value)?;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        Ok(parsed)
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
