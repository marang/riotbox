use std::path::{Path, PathBuf};
use std::process::ExitCode;

use riotbox_core::source_graph::{
    TimingFixtureEvaluation, analyze_source_timing_seed, evaluate_timing_fixture_output,
    source_timing_analysis_seed_from_fixture_case,
    timing_fixture_evaluation_target_from_fixture_case,
};
use serde::Serialize;

const SCHEMA: &str = "riotbox.source_timing_fixture_evaluation_report.v1";
const DEFAULT_CATALOG: &str =
    "crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json";

#[derive(Clone, Debug, PartialEq)]
struct Args {
    catalog_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct FixtureEvaluationReport {
    schema: &'static str,
    schema_version: u32,
    catalog_path: String,
    case_count: usize,
    passed: bool,
    evaluations: Vec<TimingFixtureEvaluation>,
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(report) => {
            match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("source timing fixture report error: {error}");
                    return ExitCode::FAILURE;
                }
            }

            if report.passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("source timing fixture report error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<FixtureEvaluationReport, String> {
    let args = parse_args(args)?;
    build_report(&args.catalog_path)
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut catalog_path = PathBuf::from(DEFAULT_CATALOG);
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--catalog" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--catalog requires a path".to_string())?;
                catalog_path = PathBuf::from(value);
            }
            "--help" | "-h" => {
                return Err(format!(
                    "usage: source_timing_fixture_report [--catalog {DEFAULT_CATALOG}]"
                ));
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(Args { catalog_path })
}

fn build_report(catalog_path: &Path) -> Result<FixtureEvaluationReport, String> {
    let catalog_text = std::fs::read_to_string(catalog_path)
        .map_err(|error| format!("failed to read {}: {error}", catalog_path.display()))?;
    let catalog: serde_json::Value = serde_json::from_str(&catalog_text)
        .map_err(|error| format!("failed to parse {}: {error}", catalog_path.display()))?;
    let cases = catalog
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "fixture catalog cases must be an array".to_string())?;

    let mut evaluations = Vec::with_capacity(cases.len());
    for case in cases {
        let seed = source_timing_analysis_seed_from_fixture_case(case)?;
        let target = timing_fixture_evaluation_target_from_fixture_case(case)?;
        let timing = analyze_source_timing_seed(&seed);
        evaluations.push(evaluate_timing_fixture_output(&timing, &target));
    }

    Ok(FixtureEvaluationReport {
        schema: SCHEMA,
        schema_version: 1,
        catalog_path: catalog_path.display().to_string(),
        case_count: evaluations.len(),
        passed: evaluations.iter().all(|evaluation| evaluation.passed),
        evaluations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_and_custom_catalog_args() {
        assert_eq!(
            parse_args(Vec::<String>::new()).expect("default args"),
            Args {
                catalog_path: PathBuf::from(DEFAULT_CATALOG)
            }
        );
        assert_eq!(
            parse_args(["--catalog".into(), "fixtures/catalog.json".into()]).expect("custom args"),
            Args {
                catalog_path: PathBuf::from("fixtures/catalog.json")
            }
        );
    }

    #[test]
    fn rejects_missing_catalog_arg_value() {
        let error = parse_args(["--catalog".into()]).expect_err("missing value");
        assert!(error.contains("--catalog requires a path"));
    }

    #[test]
    fn report_evaluates_committed_fixture_catalog() {
        let catalog_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/source_timing/timing_fixture_catalog.json");
        let report = build_report(&catalog_path).expect("fixture report");
        let report_json = serde_json::to_value(&report).expect("report json");

        assert_eq!(report_json["schema"], SCHEMA);
        assert_eq!(report_json["schema_version"], 1);
        assert_eq!(report_json["passed"], true);
        assert!(report.case_count >= 5);
        assert_eq!(
            report_json["evaluations"][0]["fixture_id"],
            "fx_timing_clean_128_4x4"
        );
        assert_eq!(report_json["evaluations"][0]["passed"], true);
        assert!(report_json["evaluations"][0]["primary_confidence"].is_number());
        assert!(report_json["evaluations"][0]["primary_max_drift_ms"].is_number());
        assert!(report_json["evaluations"][0]["issues"].is_array());
    }
}
