# P012 Timing Report Gate Periodic Review - 2026-05-10

## Scope

Scheduled cross-slice review for `RIOTBOX-736` after the recent P012 fixture
report, Markdown report, JSON validator, and object-issue serialization slices.

Reviewed:

- `crates/riotbox-core/src/source_graph/`
- `crates/riotbox-core/src/bin/source_timing_fixture_report.rs`
- `scripts/validate_source_timing_fixture_report_json.py`
- `scripts/validate_source_timing_fixture_catalog.py`
- `Justfile` source-timing fixture report recipes
- `docs/benchmarks/source_timing_fixture_evaluator_2026-05-07.md`

This review did not change timing behavior.

## Findings

### Major: direct report path silently accepts mistyped fixture labels

Location:

- `crates/riotbox-core/src/source_graph/timing_evaluation.rs:270`
- `crates/riotbox-core/src/source_graph/timing_evaluation.rs:313`
- `crates/riotbox-core/src/bin/source_timing_fixture_report.rs:99`

Category: scope / correctness

`source_timing_fixture_report` builds reports directly from the catalog but does
not enforce the strict catalog validator before converting fixture expectations.
The Rust conversion helpers currently map unknown alternative kinds to
`TimingHypothesisKind::Ambiguous` and unknown warning labels to
`TimingWarningCode::LowTimingConfidence`.

That means a typo in a custom or future fixture catalog can still produce a
passing report. The existing `just ci` path also runs the Python catalog
validator, so the committed catalog is protected, but the report binary itself is
not a self-contained gate for externally supplied catalogs.

Reproduction:

```bash
# Change fx_timing_halftime_140_ambiguous expected alternative kind from
# half_time to halff_time_typo in a temporary catalog, then run:
cargo run -q -p riotbox-core --bin source_timing_fixture_report -- --catalog "$tmp"
```

Observed result:

```text
rc=0
passed=true
issues=[]
```

A similar temporary typo from `phrase_uncertain` to `phrase_uncertain_typo` on
the weak fixture also passed because both the analysis seed and target converted
the unknown warning into the same fallback value.

Suggestion:

Make the Rust fixture conversion strict for catalog-provided enum labels, return
an error for unknown timing quality, degraded policy, warning code, or
alternative kind, and add focused tests proving `source_timing_fixture_report`
rejects invalid labels even when the Python catalog validator is not run first.

Follow-up:

- `RIOTBOX-737` should implement the strict Rust catalog label handling.

## Non-Findings

- The current file sizes in the reviewed timing/report area are within the
  soft review budget; no mechanical split is needed.
- The JSON report validator covers the current downstream report contract shape:
  schema/version, case count, aggregate pass consistency, duplicate fixture ids,
  required measurements, and known issue code/object issue variants.
- The Markdown report is correctly derived from the same `TimingFixtureEvaluation`
  data as the JSON report; no separate report model was introduced.

## Verification Commands

```bash
wc -l crates/riotbox-core/src/source_graph/*.rs \
  crates/riotbox-core/src/source_graph/*/*.rs \
  crates/riotbox-core/src/bin/source_timing_fixture_report.rs \
  scripts/validate_source_timing_fixture_report_json.py \
  docs/benchmarks/source_timing_fixture_evaluator_2026-05-07.md

rg -n "source-timing-fixture-report|source_timing_fixture_report|TimingFixtureEvaluation" \
  Justfile crates/riotbox-core/src scripts docs/benchmarks docs/specs docs/reviews
```
