use super::args::Args;
use super::case_catalog::pack_cases;
use super::config::SAMPLE_RATE;
use super::manifest::write_manifest;
use super::render_case::render_case;
use super::report_markdown::render_pack_summary;
use std::fs;

pub(super) fn render_pack(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
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
