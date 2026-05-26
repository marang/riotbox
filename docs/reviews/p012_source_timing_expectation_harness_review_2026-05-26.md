# P012 Source Timing Expectation Harness Review - 2026-05-26

## Scope

Focused `review-codebase` pass after RIOTBOX-996 through RIOTBOX-999 tightened
local Source Timing example expectations for anchor evidence, selected downbeat
offsets, groove residual counts, and unavailable null metrics.

Reviewed:

- `scripts/source_timing_example_expectations.py`
- `scripts/source_timing_example_probe_report.py`
- `scripts/assert_source_timing_example_report_fixtures.py`
- `scripts/fixtures/source_timing_example_probe_report/*expectations*.json`

This review did not change analyzer, readiness policy, Session, UI, realtime
audio, or output behavior.

## Summary

The expectation harness is coherent and now covers the major compact P012 local
example evidence surfaces: status labels, BPM/beat/downbeat score evidence,
alternate counts, selected downbeat offsets, anchor evidence, groove evidence,
warning modes, and unavailable null metrics.

The recent additions are intentionally QA-only. They make report drift visible
without claiming the detector is more mature or promoting manual-confirm rows to
locked timing.

## Findings

### Minor - Rendered downbeat margin is not asserted by row fixture checks

- Location: `scripts/source_timing_example_probe_report.py:54`
- Location: `scripts/source_timing_example_probe_report.py:201`
- Location: `scripts/source_timing_example_probe_report.py:248`
- Location: `scripts/assert_source_timing_example_report_fixtures.py:33`
- Category: scope
- Severity: minor

`ReportRow` carries `downbeat_margin`, `row_from_payload(...)` formats it, and
the Markdown renderer emits it. The committed positive row assertions in
`EXPECTED_ROWS` do not currently include the `downbeat_margin` field, so a
formatting or mapping regression for that rendered column could slip past
`just source-timing-example-probe-report-fixtures` even though expectation JSON
checks still validate the raw `primary_downbeat_margin` payload values.

Suggestion: add `downbeat_margin` entries to `EXPECTED_ROWS` for the committed
positive fixtures and keep the existing expectation-JSON margin checks as the
payload-level gate.

## Non-Findings

- Unknown top-level expectation keys still fail before comparison.
- Empty, negative, and unknown nested anchor/groove expectation shapes are
  covered by invalid fixtures.
- Null numeric metric expectations are covered by a mismatch fixture against a
  non-null row.
- The local Beat20 row remains manual-confirm-only; the harness now asserts its
  selected offset and transient-only anchor evidence without treating that as
  trusted downbeat confidence.

## Recommended Next Slice

`Assert rendered downbeat margin in example row fixtures`

Functional impact:

- no sound, UI, analyzer, Session, or policy change
- tighter committed report-fixture coverage for the downbeat-margin column
- keeps payload-level and rendered-row checks aligned

## Verification

```bash
git diff --check
```
