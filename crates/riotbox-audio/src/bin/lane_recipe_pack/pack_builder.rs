use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use riotbox_audio::{
    listening_manifest::{
        LISTENING_MANIFEST_SCHEMA_VERSION, ListeningPackArtifact as ManifestArtifact,
        ListeningPackSignalMetrics as ManifestSignalMetrics, write_manifest_json,
    },
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting,
        Mc202RenderState,
    },
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

    render_pack(&args)?;
    println!("wrote {}", args.output_dir().display());

    Ok(())
}

fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
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

    let summary = render_pack_summary(args, &output_dir, &reports);
    let summary_path = output_dir.join("pack-summary.md");
    fs::write(&summary_path, summary)?;
    write_manifest(
        &output_dir.join("manifest.json"),
        args,
        &output_dir,
        &reports,
    )?;

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
    title: &'static str,
    recipe_refs: &'static str,
    baseline_label: &'static str,
    candidate_label: &'static str,
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
            min_rms_delta: 0.0055,
            min_signal_delta_rms: 0.006,
            note: "This proves the `<` / `>` touch gesture changes the same MC-202 phrase energy rather than only changing UI state.",
        },
        PackCase {
            id: "mc202-follower-to-pressure",
            title: "MC-202 follower -> pressure",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "pressure cell",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Pressure,
                    Mc202PhraseShape::PressureCell,
                    0.84,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.004,
            note: "This proves the `P` pressure gesture renders a sparse offbeat pressure cell instead of the follower drive pattern.",
        },
        PackCase {
            id: "mc202-follower-to-instigator",
            title: "MC-202 follower -> instigator",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "instigator spike",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Instigator,
                    Mc202PhraseShape::InstigatorSpike,
                    0.90,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.009,
            note: "This proves the `I` instigate gesture renders a sharper high-register shove instead of the follower drive pattern.",
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
        PackCase {
            id: "mc202-neutral-to-lift-contour",
            title: "MC-202 neutral -> lift contour",
            recipe_refs: "Recipe 2",
            baseline_label: "follower neutral contour",
            candidate_label: "follower lift contour",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state_with_contour(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Neutral,
                ),
                candidate: mc202_state_with_contour(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Lift,
                ),
            },
            min_rms_delta: 0.0,
            min_signal_delta_rms: 0.004,
            note: "This proves the source-section contour hint changes the same MC-202 role at the render seam instead of only changing diagnostics.",
        },
        PackCase {
            id: "mc202-direct-to-hook-response",
            title: "MC-202 direct -> hook response",
            recipe_refs: "Recipe 2",
            baseline_label: "follower direct",
            candidate_label: "follower answer-space",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state_with_policy(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Neutral,
                    Mc202HookResponse::Direct,
                ),
                candidate: mc202_state_with_policy(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Neutral,
                    Mc202HookResponse::AnswerSpace,
                ),
            },
            min_rms_delta: 0.001,
            min_signal_delta_rms: 0.004,
            note: "This proves hook-response mode leaves space against the same follower phrase instead of doubling the hook downbeats.",
        },
    ]
}

