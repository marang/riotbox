# Sound Quality Readiness Report v1

Date: 2026-06-12

P023 adds a CI-safe sound-quality readiness report that aggregates existing
contracts instead of creating another taste oracle. The report reads the
sound-product rubric, source corpus, release-grade demo bank, weak-output
routing report, and optionally the current professional-output suite.

The generator writes:

```text
artifacts/audio_qa/local-sound-quality-readiness-report/sound-quality-readiness-report.json
artifacts/audio_qa/local-sound-quality-readiness-report/sound-quality-readiness-report.md
```

Run:

```bash
just sound-quality-readiness-report-smoke
```

The report exposes:

- `release_readiness`: currently `blocked` until all P023 source families have
  demo-ready human-pass coverage and no weak/unverified candidates are promoted.
- `quality_claim_allowed`: `false` while blockers remain.
- `source_family_coverage`: required P023 source families, demo-bank aliases,
  missing candidates, missing human verdicts, and missing demo-ready coverage.
- `weak_output_routing`: concrete production fix categories and artifact paths
  when the routing report is available, including P023 production-fix
  candidates and their grouped `production_fix_summary` when the routing report
  exposes them.
- `professional_output_suite`: current diagnostic context from the professional
  suite, including strongest audible elements, W-30/source hook presence,
  source-character hook/chop selection floor/span, dense drum-pressure proof,
  sparse bass-pressure movement / low-band / dominance proof, mix-balance
  summary, and the diagnostic-only quality boundary.
- `blockers`: machine-readable reasons that prevent release-ready claims.
- `next_actions`: engineer- and musician-facing work categories for the next
  sound-improvement slices.

This is a status and actionability report. It does not approve audio, replace
human listening, or promote scripted diagnostic evidence into product-quality
proof.
