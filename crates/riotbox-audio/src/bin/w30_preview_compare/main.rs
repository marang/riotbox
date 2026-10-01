mod args;
mod comparison;
mod config;
mod manifest;
mod metrics_input;
mod report_markdown;
#[cfg(test)]
mod tests;

use args::{Args, print_help};
use comparison::compare_metrics;
use manifest::{manifest_path_for_report_path, write_manifest};
use metrics_input::SmokeMetrics;
use report_markdown::{render_report, write_report_markdown};
use std::{env, process};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let baseline = SmokeMetrics::read_from_path(&args.baseline_metrics_path)?;
    let candidate = SmokeMetrics::read_from_path(&args.candidate_metrics_path)?;
    let report = compare_metrics(&baseline, &candidate, &args.limits);
    let rendered_report = render_report(
        &args.baseline_metrics_path,
        &args.candidate_metrics_path,
        &report,
    );

    println!("{rendered_report}");
    write_report_markdown(&args.report_path, &rendered_report)?;
    println!("wrote {}", args.report_path.display());
    write_manifest(&args, baseline, candidate, &report)?;
    println!(
        "wrote {}",
        manifest_path_for_report_path(&args.report_path).display()
    );

    if report.has_failures() {
        process::exit(2);
    }

    Ok(())
}
