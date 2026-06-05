# Professional Output Suite v1

`riotbox.professional_output_suite.v1` is the aggregate status report for the
current professional sound-output path.

It exists so reviewers do not have to infer product readiness from several
unrelated reports. The suite runs the current child gates together and records
their schema, result, agent verdict, human verdict, report hash, and key metrics.

Child reports:

- dense-break performance pack
- pro-pressure source matrix
- professional source WAV pack for tonal and sparse local sources
- professional output listening pack
- destructive variation professional report
- rendered weak professional-output diagnostics

The suite also verifies the structured listening pack identity:

- candidate WAV files exist
- candidate hashes match the listening-pack report
- review JSON files exist
- review hashes match the listening-pack report
- source-report hashes match the source reports beside the rendered WAVs
- dense, tonal, and sparse source families are all present

Professional listening reviews also carry `audio_judge_label` metadata for later
human verdict import. The metadata records source family/id, review-pack schema,
performance-report hash, agent-review hash, source/full-performance audio
hashes, artifact paths, and reason-tag defaults. Import still requires a recorded
human verdict; unverified packs must not become taste labels.

The suite's negative side includes `rendered-weak-professional-output-fixtures`.
That target writes actual weak WAV artifacts for a dense/destructive flat-stutter
case, builds a report from those rendered metrics, and proves the destructive
variation professional gate rejects it for concrete musical reasons: flat
dropout/stutter contrast and a restore that does not come back bigger than the
pressure section.

The suite also enforces the shared evidence-boundary contract. Current scripted
diagnostics must report `quality_proof: false`; the suite must fail if any
scripted child report claims product-quality proof. Source-backed diagnostics
may report `source_backed: true` and `source_timing_backed: true`, but only as
diagnostic evidence while the arrangement / mix decision is still scripted.
Synthetic negative fixtures report `source_backed: false`.

Run:

```bash
just professional-output-suite-smoke
just professional-output-listening-verdict-import-fixtures
just rendered-weak-professional-output-fixtures
```

Boundary:

- This is a deterministic CI-safe professional-output status surface.
- It proves the current reports are fresh, hash-bound, present, and passing
  together.
- It can prove smoke, regression, and diagnostic behavior for the current
  scripted render path, but hardcoded or scripted audio generation is not
  technical or musical quality proof for the product.
- It enforces machine-readable evidence fields: `evidence_role`,
  `source_backed`, `source_timing_backed`, `scripted_generation`,
  `quality_proof`, and `human_verdict`.
- It does not claim a human musical pass; `human_verdict` remains `unverified`
  until a structured human review is recorded.
