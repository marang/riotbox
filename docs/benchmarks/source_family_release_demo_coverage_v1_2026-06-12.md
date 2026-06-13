# Source-Family Release-Demo Coverage v1

Date: 2026-06-12

P023 adds a CI-safe coverage gate that checks the release-grade demo bank
against the P023 real-source corpus. The gate does not inspect ignored WAV
files; it reads only committed JSON contracts and reports which source families
have candidates, structured human verdicts, and demo-ready human-pass examples.

Run:

```bash
just source-family-release-demo-coverage-fixtures
```

The validator writes:

```text
artifacts/audio_qa/local-source-family-release-demo-coverage/source-family-release-demo-coverage.json
artifacts/audio_qa/local-source-family-release-demo-coverage/source-family-release-demo-coverage.md
```

The report exposes:

- `release_readiness`: `blocked` until every required P023 source family has a
  demo-ready human-pass entry.
- `source_files_required`: always `false`; this gate must work in CI without
  local ignored WAV files.
- `missing_demo_candidate_families`: source families with no demo-bank entry at
  all.
- `missing_human_verdict_families`: source families with candidates but no
  `pass`, `weak`, or `fail` human verdict.
- `missing_demo_ready_families`: source families without a demo-ready
  human-pass entry.
- `families[]`: per-family aliases, corpus cases, candidate entries,
  human-verdict entries, demo-ready entries, and status.

Candidate-only and weak/fail human-verdict families are useful implementation
evidence, but they still block release-ready claims. The gate is a coverage
contract, not a musical judge and not a quality proof.

Additional unverified candidates can be added beside an existing weak/fail
human verdict to give reviewers better material. That must not promote the
family: for example, sparse-drums coverage stays `human_verdict_non_demo` until
a structured human pass marks a sparse-bass-pressure entry `demo_ready`.
