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
    markdown_output: Option<PathBuf>,
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
    let report = build_report(&args.catalog_path)?;
    if let Some(markdown_output) = args.markdown_output {
        std::fs::write(&markdown_output, render_markdown_report(&report))
            .map_err(|error| format!("failed to write {}: {error}", markdown_output.display()))?;
    }
    Ok(report)
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut catalog_path = PathBuf::from(DEFAULT_CATALOG);
    let mut markdown_output = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--catalog" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--catalog requires a path".to_string())?;
                catalog_path = PathBuf::from(value);
            }
            "--markdown-output" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--markdown-output requires a path".to_string())?;
                markdown_output = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err(format!(
                    "usage: source_timing_fixture_report [--catalog {DEFAULT_CATALOG}] [--markdown-output report.md]"
                ));
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(Args {
        catalog_path,
        markdown_output,
    })
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

fn render_markdown_report(report: &FixtureEvaluationReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Source Timing Fixture Evaluation Report\n\n");
    markdown.push_str(&format!("- Schema: `{}`\n", report.schema));
    markdown.push_str(&format!("- Catalog: `{}`\n", report.catalog_path));
    markdown.push_str(&format!("- Cases: `{}`\n", report.case_count));
    markdown.push_str(&format!(
        "- Result: `{}`\n\n",
        if report.passed { "pass" } else { "fail" }
    ));
    markdown.push_str(
        "| Fixture | Result | BPM Error | Confidence | Mean Drift | Max Drift | Issues |\n",
    );
    markdown.push_str("|---|---:|---:|---:|---:|---:|---|\n");

    for evaluation in &report.evaluations {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{:.3}` | {} | {} | {} | {} |\n",
            evaluation.fixture_id,
            if evaluation.passed { "pass" } else { "fail" },
            evaluation.bpm_error,
            optional_f32_label(evaluation.primary_confidence),
            optional_f32_label(evaluation.primary_max_mean_abs_drift_ms),
            optional_f32_label(evaluation.primary_max_drift_ms),
            issue_label(evaluation),
        ));
    }

    markdown
}

fn optional_f32_label(value: Option<f32>) -> String {
    value
        .map(|value| format!("`{value:.3}`"))
        .unwrap_or_else(|| "`unknown`".into())
}

fn issue_label(evaluation: &TimingFixtureEvaluation) -> String {
    if evaluation.issues.is_empty() {
        return "`none`".into();
    }

    evaluation
        .issues
        .iter()
        .map(|issue| {
            serde_json::to_string(issue)
                .unwrap_or_else(|_| format!("{issue:?}"))
                .trim_matches('"')
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_and_custom_catalog_args() {
        assert_eq!(
            parse_args(Vec::<String>::new()).expect("default args"),
            Args {
                catalog_path: PathBuf::from(DEFAULT_CATALOG),
                markdown_output: None,
            }
        );
        assert_eq!(
            parse_args([
                "--catalog".into(),
                "fixtures/catalog.json".into(),
                "--markdown-output".into(),
                "report.md".into(),
            ])
            .expect("custom args"),
            Args {
                catalog_path: PathBuf::from("fixtures/catalog.json"),
                markdown_output: Some(PathBuf::from("report.md")),
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

    #[test]
    fn markdown_report_uses_json_report_data() {
        let report = FixtureEvaluationReport {
            schema: SCHEMA,
            schema_version: 1,
            catalog_path: "fixtures/catalog.json".into(),
            case_count: 1,
            passed: true,
            evaluations: vec![TimingFixtureEvaluation {
                fixture_id: "fx_timing_clean_128_4x4".into(),
                passed: true,
                bpm_error: 0.397,
                beat_count: 32,
                bar_count: 8,
                phrase_count: 2,
                primary_confidence: Some(0.85),
                primary_max_mean_abs_drift_ms: Some(17.5),
                primary_max_drift_ms: Some(35.0),
                issues: vec![],
            }],
        };

        let markdown = render_markdown_report(&report);

        assert!(markdown.contains("# Source Timing Fixture Evaluation Report"));
        assert!(markdown.contains("fx_timing_clean_128_4x4"));
        assert!(markdown.contains("| `fx_timing_clean_128_4x4` | `pass` |"));
        assert!(markdown.contains("`none`"));
    }

    #[test]
    fn report_rejects_unknown_warning_label() {
        let mut catalog = committed_catalog();
        let weak_case = catalog["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|case| case["fixture_id"] == "fx_timing_weak_noisy_123")
            .expect("weak fixture");
        weak_case["expected"]["warnings"][2] = serde_json::json!("phrase_uncertain_typo");
        let path = write_temp_catalog("unknown-warning", &catalog);

        let error = build_report(&path).expect_err("unknown warning rejected");

        let _ = std::fs::remove_file(&path);
        assert!(error.contains("fx_timing_weak_noisy_123"));
        assert!(error.contains("unknown warning label"));
        assert!(error.contains("phrase_uncertain_typo"));
    }

    #[test]
    fn report_rejects_unknown_timing_quality_label() {
        let mut catalog = committed_catalog();
        let clean_case = catalog["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|case| case["fixture_id"] == "fx_timing_clean_128_4x4")
            .expect("clean fixture");
        clean_case["expected"]["timing_quality"] = serde_json::json!("pretty_good");
        let path = write_temp_catalog("unknown-quality", &catalog);

        let error = build_report(&path).expect_err("unknown timing quality rejected");

        let _ = std::fs::remove_file(&path);
        assert!(error.contains("fx_timing_clean_128_4x4"));
        assert!(error.contains("unknown timing_quality"));
        assert!(error.contains("pretty_good"));
    }

    #[test]
    fn report_rejects_unknown_degraded_policy_label() {
        let mut catalog = committed_catalog();
        let clean_case = catalog["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|case| case["fixture_id"] == "fx_timing_clean_128_4x4")
            .expect("clean fixture");
        clean_case["expected"]["degraded_policy"] = serde_json::json!("mostly_locked");
        let path = write_temp_catalog("unknown-policy", &catalog);

        let error = build_report(&path).expect_err("unknown degraded policy rejected");

        let _ = std::fs::remove_file(&path);
        assert!(error.contains("fx_timing_clean_128_4x4"));
        assert!(error.contains("unknown degraded_policy"));
        assert!(error.contains("mostly_locked"));
    }

    #[test]
    fn report_rejects_unknown_alternative_kind() {
        let mut catalog = committed_catalog();
        let ambiguous_case = catalog["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|case| case["fixture_id"] == "fx_timing_halftime_140_ambiguous")
            .expect("ambiguous fixture");
        ambiguous_case["expected"]["alternatives"][0]["kind"] =
            serde_json::json!("halff_time_typo");
        let path = write_temp_catalog("unknown-alternative", &catalog);

        let error = build_report(&path).expect_err("unknown alternative rejected");

        let _ = std::fs::remove_file(&path);
        assert!(error.contains("fx_timing_halftime_140_ambiguous"));
        assert!(error.contains("unknown alternative kind"));
        assert!(error.contains("halff_time_typo"));
    }

    fn committed_catalog() -> serde_json::Value {
        let catalog_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/source_timing/timing_fixture_catalog.json");
        let catalog_text = std::fs::read_to_string(catalog_path).expect("read catalog");
        serde_json::from_str(&catalog_text).expect("parse catalog")
    }

    fn write_temp_catalog(name: &str, catalog: &serde_json::Value) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "riotbox-source-timing-fixture-{name}-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            serde_json::to_string_pretty(catalog).expect("serialize catalog"),
        )
        .expect("write temp catalog");
        path
    }
}
