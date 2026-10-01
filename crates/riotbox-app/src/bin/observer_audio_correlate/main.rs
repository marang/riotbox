use args::Args;
use args::print_help;
use metadata_io::read_observer_events;
use metadata_io::validate_manifest_envelope_file;
use observer_validation::validate_user_session_observer_events;
use std::env;
use std::fs;
use summary_build::build_summary_from_events;
use summary_evidence::validate_required_evidence;
use summary_json::render_json;
use summary_markdown::render_markdown;

mod args;
mod lane_recipe_output;
mod manifest_metrics;
mod manifest_source_timing;
mod metadata_io;
mod observer_events;
mod observer_source_timing;
mod observer_validation;
mod report_model;
mod scene_movement;
mod source_timing_alignment;
mod source_timing_anchor_evidence;
mod source_timing_groove_evidence;
mod source_timing_labels;
mod source_timing_policy;
mod summary_build;
mod summary_evidence;
mod summary_json;
mod summary_markdown;
mod value_fields;

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

#[cfg(test)]
mod lane_recipe_tests;
#[cfg(test)]
mod observer_source_timing_tests;
#[cfg(test)]
mod scene_movement_tests;
#[cfg(test)]
mod source_grid_output_drift_tests;
#[cfg(test)]
mod source_timing_alignment_tests;
#[cfg(test)]
mod source_timing_evidence_tests;
#[cfg(test)]
mod summary_smoke_tests;
#[cfg(test)]
mod tests;
