# P012 Source Timing Report Expectation Gate Review - 2026-05-21

## Scope

Broad-review cadence pass after RIOTBOX-821 through RIOTBOX-827 tightened the
Source Timing example report expectation and fixture gates.

Reviewed:

- `scripts/source_timing_example_probe_report.py`
- `scripts/source_timing_example_expectations.py`
- `scripts/assert_source_timing_example_report_fixtures.py`
- `scripts/fixtures/source_timing_example_probe_report/`
- example-report contract text in
  `docs/specs/source_timing_intelligence_spec.md`

This review did not change analyzer, Session, action, JamAppState, or
audio-output behavior.

## Findings

### Major: CLI reports malformed fixture shape with a traceback instead of a bounded error

Location:

- `scripts/source_timing_example_probe_report.py:100`
- `scripts/source_timing_example_probe_report.py:154`
- `scripts/source_timing_example_probe_report.py:291`
- `scripts/source_timing_example_probe_report.py:298`
- `scripts/source_timing_example_probe_report.py:305`

Category: scope / QA ergonomics

`main()` catches `OSError`, `ValueError`, and `subprocess.CalledProcessError`,
but the report's local shape guards raise `TypeError` for malformed probe or
expectation JSON. A bad fixture such as a missing string field can therefore
escape the existing friendly `source timing example probe report error: ...`
path and print a Python traceback.

Impact:

The CI target still fails, but the failure mode is noisier and less consistent
than the rest of the Source Timing fixture gates. That matters because this
report is intentionally a reviewer-facing QA surface.

Suggestion:

Catch `TypeError` in the report CLI's top-level error path and add a tiny
invalid fixture or smoke proving malformed fixture JSON exits through the
bounded error message.

### Major: expectation warnings can only be inclusion-checked, not exact-checked

Location:

- `scripts/source_timing_example_expectations.py:192`
- `scripts/source_timing_example_expectations.py:199`
- `scripts/fixtures/source_timing_example_probe_report/beat08_expectations.json:63`
- `scripts/fixtures/source_timing_example_probe_report/local_example_expectations.json:26`
- `scripts/fixtures/source_timing_example_probe_report/local_example_expectations.json:63`

Category: scope / correctness

The expectation contract supports `warning_codes_include`, which is useful for
degraded or ambiguous rows. It cannot express "there must be no warnings" or
"the warnings must be exactly this set." In practice, an expectation entry with
`"warning_codes_include": []` passes even if the payload grows unexpected warning
codes.

Impact:

The committed field assertor currently catches warnings for its hardcoded rows,
but optional local example expectations rely on the expectation comparator. A
stable local row could gain an unexpected warning and still report `ok` if every
other field remains inside tolerance.

Suggestion:

Add a strict warning expectation such as `warning_codes_exact`, reject mixing it
with `warning_codes_include`, and add positive/negative fixtures that prove
warning-free locked rows and expected degraded warning rows are checked exactly
when the stricter key is used.

## Non-Findings

- Unknown top-level expectation keys now fail before comparison, which closes
  the typo risk identified after RIOTBOX-826.
- Numeric range expectations now reject empty ranges, inverted ranges, and
  unknown range keys.
- Positive committed report fixtures now validate against the Source Timing
  probe JSON schema before field-level report assertions.
- The report and expectation helper files remain under the current soft review
  budget after the split.

## Recommended Follow-Ups

1. Catch `TypeError` in the Source Timing example report CLI and add malformed
   fixture coverage for the bounded error path.
2. Add exact warning-code expectations for Source Timing example report
   expectations.

## Verification Commands

```bash
nl -ba scripts/source_timing_example_probe_report.py | sed -n '1,340p'
nl -ba scripts/source_timing_example_expectations.py | sed -n '1,260p'
nl -ba scripts/assert_source_timing_example_report_fixtures.py | sed -n '1,260p'
find scripts/fixtures/source_timing_example_probe_report -maxdepth 1 -type f -name '*.json' -print | sort
```
