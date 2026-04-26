use std::{
    env, fs,
    path::{Path, PathBuf},
};

use riotbox_audio::{
    mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState},
    runtime::{OfflineAudioMetrics, render_mc202_offline, render_tr909_offline, signal_metrics},
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
        Tr909TakeoverRenderProfile,
    },
};

const SAMPLE_RATE: u32 = 44_100;
const CHANNEL_COUNT: u16 = 2;
const DEFAULT_DURATION_SECONDS: f32 = 2.0;
const PACK_ID: &str = "lane-recipe-listening-pack";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let output_dir = args.output_dir();
    fs::create_dir_all(&output_dir)?;

    let frame_count = (args.duration_seconds * SAMPLE_RATE as f32).round() as usize;
    let mut reports = Vec::new();

    for case in pack_cases() {
        reports.push(render_case(
            &output_dir,
            case,
            frame_count,
            args.duration_seconds,
        )?);
    }

    let summary = render_pack_summary(&args, &output_dir, &reports);
    let summary_path = output_dir.join("pack-summary.md");
    fs::write(&summary_path, summary)?;

    println!("wrote {}", output_dir.display());
    println!("wrote {}", summary_path.display());

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct Args {
    date: String,
    output_dir: Option<PathBuf>,
    duration_seconds: f32,
    show_help: bool,
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
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
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

    fn output_dir(&self) -> PathBuf {
        self.output_dir.clone().unwrap_or_else(|| {
            Path::new("artifacts")
                .join("audio_qa")
                .join(&self.date)
                .join(PACK_ID)
        })
    }
}

fn print_help() {
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

#[derive(Clone, Debug)]
struct PackCase {
    id: &'static str,
    title: &'static str,
    recipe_refs: &'static str,
    baseline_label: &'static str,
    candidate_label: &'static str,
    render_pair: RenderPair,
    min_rms_delta: f32,
    min_signal_delta_rms: f32,
    note: &'static str,
}

#[derive(Clone, Debug)]
enum RenderPair {
    Tr909 {
        baseline: Tr909RenderState,
        candidate: Tr909RenderState,
    },
    Mc202 {
        baseline: Mc202RenderState,
        candidate: Mc202RenderState,
    },
}

#[derive(Debug)]
struct CaseReport {
    id: &'static str,
    baseline_metrics: OfflineAudioMetrics,
    candidate_metrics: OfflineAudioMetrics,
    signal_delta_metrics: OfflineAudioMetrics,
    min_rms_delta: f32,
    min_signal_delta_rms: f32,
    passed: bool,
}

fn pack_cases() -> Vec<PackCase> {
    vec![
        PackCase {
            id: "tr909-support-to-fill",
            title: "TR-909 support -> fill",
            recipe_refs: "Recipe 2, Recipe 7",
            baseline_label: "steady source support",
            candidate_label: "fill with mainline drive",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::SteadyPulse,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: Tr909RenderState {
                    mode: Tr909RenderMode::Fill,
                    routing: Tr909RenderRouting::DrumBusSupport,
                    pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
                    phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
                    drum_bus_level: 0.82,
                    is_transport_running: true,
                    tempo_bpm: 128.0,
                    position_beats: 32.0,
                    ..Tr909RenderState::default()
                },
            },
            min_rms_delta: 0.001,
            min_signal_delta_rms: 0.001,
            note: "The fill candidate should be busier and more assertive than steady support.",
        },
        PackCase {
            id: "tr909-support-to-takeover",
            title: "TR-909 support -> takeover",
            recipe_refs: "Recipe 2",
            baseline_label: "steady source support",
            candidate_label: "controlled phrase takeover",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: Tr909RenderState {
                    mode: Tr909RenderMode::Takeover,
                    routing: Tr909RenderRouting::DrumBusTakeover,
                    pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
                    phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
                    takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
                    drum_bus_level: 0.86,
                    slam_intensity: 0.3,
                    is_transport_running: true,
                    tempo_bpm: 128.0,
                    position_beats: 32.0,
                    ..Tr909RenderState::default()
                },
            },
            min_rms_delta: 0.004,
            min_signal_delta_rms: 0.004,
            note: "The takeover candidate should be more forward than support without implying a finished performance mix.",
        },
        PackCase {
            id: "scene-transport-to-target-support",
            title: "Scene transport-bar support -> scene-target support",
            recipe_refs: "Recipe 10",
            baseline_label: "transport-bar support",
            candidate_label: "scene-target support accent",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::SceneTarget,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
            },
            min_rms_delta: 0.00005,
            min_signal_delta_rms: 0.00005,
            note: "The Scene-target candidate is intentionally subtle; it proves the current TR-909 support-accent seam, not a finished Scene transition engine.",
        },
        PackCase {
            id: "mc202-follower-to-answer",
            title: "MC-202 follower -> answer",
            recipe_refs: "Recipe 2, Recipe 5",
            baseline_label: "follower drive",
            candidate_label: "answer hook",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.62,
                ),
                candidate: mc202_state(Mc202RenderMode::Answer, Mc202PhraseShape::AnswerHook, 0.78),
            },
            min_rms_delta: 0.001,
            min_signal_delta_rms: 0.005,
            note: "This is the first explicit MC-202 offline audio seam. It proves a renderable follower-vs-answer contrast, not live TUI mixer integration.",
        },
        PackCase {
            id: "mc202-touch-low-to-high",
            title: "MC-202 touch low -> high",
            recipe_refs: "Recipe 2",
            baseline_label: "follower low touch",
            candidate_label: "follower high touch",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.12,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.92,
                ),
            },
            min_rms_delta: 0.006,
            min_signal_delta_rms: 0.006,
            note: "This proves the `<` / `>` touch gesture changes the same MC-202 phrase energy rather than only changing UI state.",
        },
        PackCase {
            id: "mc202-follower-to-mutated-drive",
            title: "MC-202 follower -> mutated drive",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "mutated drive",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::MutatedDrive,
                    0.88,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.005,
            note: "This proves the `G` phrase mutation gesture produces a different rendered phrase, not an identical fallback tone at similar loudness.",
        },
    ]
}

fn tr909_support_state(
    profile: Tr909SourceSupportProfile,
    context: Tr909SourceSupportContext,
    adoption: Tr909PatternAdoption,
    variation: Tr909PhraseVariation,
) -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(profile),
        source_support_context: Some(context),
        pattern_adoption: Some(adoption),
        phrase_variation: Some(variation),
        drum_bus_level: 0.72,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        current_scene_id: (context == Tr909SourceSupportContext::SceneTarget)
            .then(|| "scene-02-break".into()),
        ..Tr909RenderState::default()
    }
}

fn mc202_state(mode: Mc202RenderMode, shape: Mc202PhraseShape, touch: f32) -> Mc202RenderState {
    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: shape,
        touch,
        music_bus_level: 0.74,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

fn render_case(
    output_dir: &Path,
    case: PackCase,
    frame_count: usize,
    duration_seconds: f32,
) -> Result<CaseReport, Box<dyn std::error::Error>> {
    let case_dir = output_dir.join(case.id);
    fs::create_dir_all(&case_dir)?;

    let (baseline, candidate) = render_pair(&case.render_pair, frame_count);
    let baseline_metrics = signal_metrics(&baseline);
    let candidate_metrics = signal_metrics(&candidate);
    let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);
    let report = CaseReport {
        id: case.id,
        baseline_metrics,
        candidate_metrics,
        signal_delta_metrics,
        min_rms_delta: case.min_rms_delta,
        min_signal_delta_rms: case.min_signal_delta_rms,
        passed: rms_delta(baseline_metrics, candidate_metrics) >= case.min_rms_delta
            && signal_delta_metrics.rms >= case.min_signal_delta_rms,
    };

    let baseline_path = case_dir.join("baseline.wav");
    let candidate_path = case_dir.join("candidate.wav");
    write_pcm16_wav(&baseline_path, SAMPLE_RATE, CHANNEL_COUNT, &baseline)?;
    write_pcm16_wav(&candidate_path, SAMPLE_RATE, CHANNEL_COUNT, &candidate)?;

    fs::write(
        case_dir.join("baseline.metrics.md"),
        render_metrics_markdown(&case, "baseline", duration_seconds, baseline_metrics),
    )?;
    fs::write(
        case_dir.join("candidate.metrics.md"),
        render_metrics_markdown(&case, "candidate", duration_seconds, candidate_metrics),
    )?;
    fs::write(
        case_dir.join("comparison.md"),
        render_comparison_markdown(&case, &report),
    )?;

    if !report.passed {
        return Err(format!(
            "{} output delta failed: RMS delta {:.6} / min {:.6}, signal delta RMS {:.6} / min {:.6}",
            report.id,
            rms_delta(report.baseline_metrics, report.candidate_metrics),
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms
        )
        .into());
    }

    Ok(report)
}

fn render_pair(render_pair: &RenderPair, frame_count: usize) -> (Vec<f32>, Vec<f32>) {
    match render_pair {
        RenderPair::Tr909 {
            baseline,
            candidate,
        } => (
            render_tr909_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_tr909_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
        RenderPair::Mc202 {
            baseline,
            candidate,
        } => (
            render_mc202_offline(baseline, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
            render_mc202_offline(candidate, SAMPLE_RATE, CHANNEL_COUNT, frame_count),
        ),
    }
}

fn render_metrics_markdown(
    case: &PackCase,
    role: &str,
    duration_seconds: f32,
    metrics: OfflineAudioMetrics,
) -> String {
    let label = if role == "baseline" {
        case.baseline_label
    } else {
        case.candidate_label
    };
    format!(
        "# Lane Recipe Listening Metrics\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Role: `{role}`\n\
         - Label: `{label}`\n\
         - Sample rate: `{SAMPLE_RATE}`\n\
         - Channels: `{CHANNEL_COUNT}`\n\
         - Duration seconds: `{duration_seconds:.3}`\n\
         - Active samples: `{}`\n\
         - Peak abs: `{:.6}`\n\
         - RMS: `{:.6}`\n\
         - Sum: `{:.6}`\n",
        case.id,
        case.title,
        case.recipe_refs,
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms,
        metrics.sum
    )
}

fn render_comparison_markdown(case: &PackCase, report: &CaseReport) -> String {
    let baseline = report.baseline_metrics;
    let candidate = report.candidate_metrics;
    let active_delta = baseline.active_samples.abs_diff(candidate.active_samples);
    let peak_delta = (baseline.peak_abs - candidate.peak_abs).abs();
    let rms_delta = rms_delta(baseline, candidate);
    let sum_delta = (baseline.sum - candidate.sum).abs();
    let signal_delta = report.signal_delta_metrics;

    format!(
        "# Lane Recipe Listening Comparison\n\n\
         - Pack: `{PACK_ID}`\n\
         - Case: `{}`\n\
         - Title: `{}`\n\
         - Recipes: `{}`\n\
         - Baseline: `{}`\n\
         - Candidate: `{}`\n\
         - Minimum RMS delta: `{:.6}`\n\
         - Signal delta RMS: `{:.6}`\n\
         - Minimum signal delta RMS: `{:.6}`\n\
         - Signal delta peak abs: `{:.6}`\n\
         - Result: `{}`\n\
         - Note: {}\n\n\
         | Metric | Baseline | Candidate | Delta |\n\
         | --- | ---: | ---: | ---: |\n\
         | active_samples | {} | {} | {} |\n\
         | peak_abs | {:.6} | {:.6} | {:.6} |\n\
         | rms | {:.6} | {:.6} | {:.6} |\n\
         | sum | {:.6} | {:.6} | {:.6} |\n",
        case.id,
        case.title,
        case.recipe_refs,
        case.baseline_label,
        case.candidate_label,
        report.min_rms_delta,
        signal_delta.rms,
        report.min_signal_delta_rms,
        signal_delta.peak_abs,
        if report.passed { "pass" } else { "fail" },
        case.note,
        baseline.active_samples,
        candidate.active_samples,
        active_delta,
        baseline.peak_abs,
        candidate.peak_abs,
        peak_delta,
        baseline.rms,
        candidate.rms,
        rms_delta,
        baseline.sum,
        candidate.sum,
        sum_delta
    )
}

fn rms_delta(baseline: OfflineAudioMetrics, candidate: OfflineAudioMetrics) -> f32 {
    (baseline.rms - candidate.rms).abs()
}

fn signal_delta_metrics(baseline: &[f32], candidate: &[f32]) -> OfflineAudioMetrics {
    debug_assert_eq!(
        baseline.len(),
        candidate.len(),
        "baseline and candidate renders should use the same frame count"
    );
    let delta = baseline
        .iter()
        .zip(candidate.iter())
        .map(|(baseline, candidate)| baseline - candidate)
        .collect::<Vec<_>>();
    signal_metrics(&delta)
}

fn render_pack_summary(args: &Args, output_dir: &Path, reports: &[CaseReport]) -> String {
    let mut summary = format!(
        "# Lane Recipe Listening Pack\n\n\
         - Pack: `{PACK_ID}`\n\
         - Date: `{}`\n\
         - Output dir: `{}`\n\
         - Duration seconds: `{:.3}`\n\n\
         This pack is the first local recipe-level audio proof outside the W-30 source-preview path.\n\
         It renders bounded TR-909, MC-202, and Scene-coupled support comparisons as WAV files plus sibling metrics.\n\n\
         ## Cases\n\n\
         | Case | Active delta | Peak delta | RMS delta | Min RMS delta | Signal delta RMS | Min signal delta RMS | Sum delta |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
        args.date,
        output_dir.display(),
        args.duration_seconds
    );

    for report in reports {
        let active_delta = report
            .baseline_metrics
            .active_samples
            .abs_diff(report.candidate_metrics.active_samples);
        let peak_delta =
            (report.baseline_metrics.peak_abs - report.candidate_metrics.peak_abs).abs();
        let rms_delta = rms_delta(report.baseline_metrics, report.candidate_metrics);
        let sum_delta = (report.baseline_metrics.sum - report.candidate_metrics.sum).abs();
        summary.push_str(&format!(
            "| `{}` | {} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            report.id,
            active_delta,
            peak_delta,
            rms_delta,
            report.min_rms_delta,
            report.signal_delta_metrics.rms,
            report.min_signal_delta_rms,
            sum_delta
        ));
    }

    summary.push_str(
        "\n## Current MC-202 Status\n\n\
         MC-202 now has explicit offline audio cases for follower-vs-answer, touch energy, and phrase mutation. These cases prove bounded renderable contrasts for the current `g`, `a`, `G`, `<`, and `>` gestures, not a finished MC-202 synth engine.\n\n\
         ## Current Scene Status\n\n\
         Scene Brain is represented here only through the current TR-909 `scene_target` support-accent seam. This does not claim a finished Scene transition engine.\n",
    );

    summary
}

fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let data_len = samples.len() * 2;
    let riff_len = 36 + data_len;
    let byte_rate = sample_rate * u32::from(channel_count) * 2;
    let block_align = channel_count * 2;

    let mut bytes = Vec::with_capacity(44 + data_len);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(riff_len as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Args, PACK_ID, pack_cases, render_pair, signal_delta_metrics, signal_metrics};
    use std::path::PathBuf;

    #[test]
    fn parses_default_args() {
        let args = Args::parse(Vec::<String>::new()).expect("parse args");

        assert_eq!(args.date, "local");
        assert_eq!(args.output_dir, None);
        assert_eq!(args.duration_seconds, 2.0);
        assert!(!args.show_help);
        assert_eq!(
            args.output_dir(),
            PathBuf::from("artifacts/audio_qa/local").join(PACK_ID)
        );
    }

    #[test]
    fn parses_custom_args() {
        let args = Args::parse([
            "--date".to_string(),
            "audit".to_string(),
            "--duration-seconds".to_string(),
            "1.5".to_string(),
            "--output-dir".to_string(),
            "tmp/pack".to_string(),
        ])
        .expect("parse args");

        assert_eq!(args.date, "audit");
        assert_eq!(args.duration_seconds, 1.5);
        assert_eq!(args.output_dir(), PathBuf::from("tmp/pack"));
    }

    #[test]
    fn rejects_invalid_duration() {
        assert!(Args::parse(["--duration-seconds".to_string(), "0".to_string()]).is_err());
    }

    #[test]
    fn pack_cases_produce_distinct_audio_metrics() {
        let cases = pack_cases();

        assert_eq!(cases.len(), 6);
        for case in cases {
            let (baseline, candidate) = render_pair(&case.render_pair, 88_200);
            let baseline_metrics = signal_metrics(&baseline);
            let candidate_metrics = signal_metrics(&candidate);
            let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);

            assert!(
                baseline_metrics.active_samples > 0,
                "{} baseline silent",
                case.id
            );
            assert!(
                candidate_metrics.active_samples > 0,
                "{} candidate silent",
                case.id
            );
            assert!(
                (baseline_metrics.rms - candidate_metrics.rms).abs() >= case.min_rms_delta,
                "{} did not produce required RMS delta {}",
                case.id,
                case.min_rms_delta
            );
            assert!(
                signal_delta_metrics.rms >= case.min_signal_delta_rms,
                "{} did not produce required signal delta RMS {}",
                case.id,
                case.min_signal_delta_rms
            );
        }
    }
}
