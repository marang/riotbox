use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use riotbox_audio::listening_manifest::validate_manifest_envelope;
use serde_json::Value;

#[path = "observer_audio_correlate/lane_recipe_output.rs"]
mod lane_recipe_output;
#[path = "observer_audio_correlate/observer_validation.rs"]
mod observer_validation;

use lane_recipe_output::{
    LaneRecipeCaseEvidence, collect_lane_recipe_cases, lane_recipe_metric_failures,
};
use observer_validation::validate_user_session_observer_events;

const STRICT_OUTPUT_METRIC_FLOOR: f64 = 1.0e-6;
const SOURCE_GRID_OUTPUT_MIN_HIT_RATIO: f64 = 0.5;
const SOURCE_TIMING_BPM_ALIGNMENT_TOLERANCE: f64 = 1.0;
const SUMMARY_SCHEMA: &str = "riotbox.observer_audio_summary.v1";
const SUMMARY_SCHEMA_VERSION: u32 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let observer_events = read_observer_events(&args.observer_path)?;
    if args.require_evidence {
        validate_user_session_observer_events(&observer_events)?;
        validate_manifest_envelope_file(&args.manifest_path)?;
    }

    let summary = build_summary_from_events(&observer_events, &args.manifest_path)?;
    let output = if args.json_output {
        render_json(&summary)?
    } else {
        render_markdown(&summary)
    };
    if args.require_evidence {
        validate_required_evidence(&summary)?;
    }

    match args.output_path {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, output)?;
        }
        None => print!("{output}"),
    }

    Ok(())
}

include!("observer_audio_correlate/args.rs");
include!("observer_audio_correlate/source_timing_anchor_evidence.rs");
include!("observer_audio_correlate/observer_source_timing.rs");
include!("observer_audio_correlate/summary_build.rs");
include!("observer_audio_correlate/source_timing_alignment.rs");
include!("observer_audio_correlate/source_timing_policy.rs");
include!("observer_audio_correlate/observer_source_timing_render.rs");
include!("observer_audio_correlate/summary_render.rs");
include!("observer_audio_correlate/summary_evidence.rs");

#[cfg(test)]
#[path = "observer_audio_correlate/lane_recipe_tests.rs"]
mod lane_recipe_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/observer_source_timing_tests.rs"]
mod observer_source_timing_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/source_grid_output_drift_tests.rs"]
mod source_grid_output_drift_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/source_timing_alignment_tests.rs"]
mod source_timing_alignment_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/source_timing_evidence_tests.rs"]
mod source_timing_evidence_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/summary_smoke_tests.rs"]
mod summary_smoke_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/tests.rs"]
mod tests;
