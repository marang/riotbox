use super::config::{
    CASE_ID, DEFAULT_DATE, DEFAULT_DURATION_SECONDS, DEFAULT_SOURCE_DURATION_SECONDS,
    DEFAULT_SOURCE_START_SECONDS, PACK_ID,
};
use super::source_window::{source_preview_from_interleaved, synthetic_source_window_preview};
use riotbox_audio::{source_audio::SourceAudioCache, w30::W30PreviewSampleWindow};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub(super) struct Args {
    pub(super) output_path: PathBuf,
    pub(super) duration_seconds: f32,
    pub(super) date: String,
    pub(super) role: RenderRole,
    pub(super) source_path: Option<PathBuf>,
    pub(super) source_start_seconds: f32,
    pub(super) source_duration_seconds: f32,
    pub(super) show_help: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RenderRole {
    Baseline,
    Candidate,
}

impl RenderRole {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "baseline" => Ok(Self::Baseline),
            "candidate" => Ok(Self::Candidate),
            other => Err(format!("unsupported role: {other}")),
        }
    }

    const fn file_stem(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }
}

impl Args {
    pub(super) fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut output_override = None;
        let mut duration_seconds = DEFAULT_DURATION_SECONDS;
        let mut date = DEFAULT_DATE.to_string();
        let mut role = RenderRole::Candidate;
        let mut source_path = None;
        let mut source_start_seconds = DEFAULT_SOURCE_START_SECONDS;
        let mut source_duration_seconds = DEFAULT_SOURCE_DURATION_SECONDS;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--out" => {
                    let Some(value) = args.next() else {
                        return Err("--out requires a path".into());
                    };
                    output_override = Some(PathBuf::from(value));
                }
                "--date" => {
                    let Some(value) = args.next() else {
                        return Err("--date requires a value".into());
                    };
                    date = value;
                }
                "--role" => {
                    let Some(value) = args.next() else {
                        return Err("--role requires a value".into());
                    };
                    role = RenderRole::parse(&value)?;
                }
                "--source" => {
                    let Some(value) = args.next() else {
                        return Err("--source requires a path".into());
                    };
                    source_path = Some(PathBuf::from(value));
                }
                "--source-start-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--source-start-seconds requires a value".into());
                    };
                    source_start_seconds =
                        parse_non_negative_seconds("--source-start-seconds", &value)?;
                }
                "--source-duration-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--source-duration-seconds requires a value".into());
                    };
                    source_duration_seconds =
                        parse_positive_seconds("--source-duration-seconds", &value)?;
                }
                "--duration-seconds" => {
                    let Some(value) = args.next() else {
                        return Err("--duration-seconds requires a value".into());
                    };
                    duration_seconds = value
                        .parse::<f32>()
                        .map_err(|_| "--duration-seconds must be a number".to_string())?;
                    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
                        return Err("--duration-seconds must be greater than zero".into());
                    }
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let output_path = output_override.unwrap_or_else(|| convention_output_path(&date, role));

        Ok(Self {
            output_path,
            duration_seconds,
            date,
            role,
            source_path,
            source_start_seconds,
            source_duration_seconds,
            show_help,
        })
    }

    pub(super) fn source_window_preview(
        &self,
    ) -> Result<W30PreviewSampleWindow, Box<dyn std::error::Error>> {
        let Some(source_path) = self.source_path.as_ref() else {
            return Ok(synthetic_source_window_preview());
        };

        let cache = SourceAudioCache::load_pcm_wav(source_path)?;
        let window =
            cache.window_by_seconds(self.source_start_seconds, self.source_duration_seconds);
        let samples = cache.window_samples(window);

        source_preview_from_interleaved(
            samples,
            usize::from(cache.channel_count),
            u64::try_from(window.start_frame).unwrap_or(u64::MAX),
            u64::try_from(window.start_frame.saturating_add(window.frame_count))
                .unwrap_or(u64::MAX),
        )
        .ok_or_else(|| {
            format!(
                "source window {} + {}s produced no samples",
                self.source_start_seconds, self.source_duration_seconds
            )
            .into()
        })
    }

    pub(super) fn source_input_label(&self) -> String {
        self.source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "synthetic".to_string())
    }
}

pub(super) fn print_help() {
    println!(
        "Usage: w30_preview_render [--date YYYY-MM-DD|local] [--role baseline|candidate] [--out PATH] [--duration-seconds SECONDS]\n\
         \n\
         Optional source-backed preview input:\n\
           --source PATH\n\
           --source-start-seconds SECONDS\n\
           --source-duration-seconds SECONDS\n\
         \n\
         Renders the initial w30-preview-smoke source-window case to a PCM16 WAV\n\
         plus a sibling metrics Markdown file. This is a local review helper,\n\
         not a full listening-pack harness yet."
    );
}

fn convention_output_path(date: &str, role: RenderRole) -> PathBuf {
    let mut path = PathBuf::from("artifacts");
    path.push("audio_qa");
    path.push(date);
    path.push(PACK_ID);
    path.push(CASE_ID);
    path.push(format!("{}.wav", role.file_stem()));
    path
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
